NIX = nix --extra-experimental-features "nix-command flakes"

.PHONY: test build smoke-test all install-hooks

## Run cargo unit tests
test:
	$(NIX) develop -c cargo test --all-features -- --test-threads=1

## Build the Docker image via Nix
build:
	$(NIX) build .#docker-image
	docker load < result

## Run smoke test against the already-loaded packet-browser:latest image
smoke-test:
	@echo "=== Smoke test: verifying Chromium starts in container ==="
	@mkdir -p /tmp/smoke-logs
	@touch /tmp/smoke-hosts

	@docker run -d --name smoke-test \
	  --read-only \
	  --tmpfs /tmp:size=128M,mode=1777 \
	  -p 127.0.0.1:63004:63004 \
	  -v /tmp/smoke-logs:/var/log/packet-browser \
	  -v /tmp/smoke-hosts:/etc/hosts \
	  --cap-drop ALL \
	  --cap-add NET_RAW \
	  -e DEBUG_MODE=true \
	  -e BLOCKLIST_ENABLED=false \
	  packet-browser:latest

	@echo "Waiting for packet-browser to start..."
	@timeout 30 bash -c 'until nc -z 127.0.0.1 63004 2>/dev/null; do sleep 1; done' \
	  || (docker logs smoke-test; docker rm -f smoke-test; echo "FAIL: service did not start"; exit 1)

	@{ sleep 1; echo "W1TEST"; sleep 1; echo "AGREE"; sleep 120; } \
	  | nc 127.0.0.1 63004 >/dev/null 2>&1 & echo $$! > /tmp/smoke-nc.pid

	@echo "Waiting for Chrome DevTools connection (up to 120s)..."
	@RESULT=1; \
	for i in $$(seq 1 120); do \
	  if docker logs smoke-test 2>&1 | grep -q '\[BROWSER\] Connected to Chrome DevTools'; then \
	    echo "PASS: Chromium started after $${i}s"; RESULT=0; break; \
	  fi; \
	  sleep 1; \
	done; \
	echo "--- Container logs ---"; \
	docker logs smoke-test 2>&1; \
	kill $$(cat /tmp/smoke-nc.pid) 2>/dev/null || true; \
	docker stop smoke-test 2>/dev/null || true; \
	docker rm smoke-test 2>/dev/null || true; \
	exit $$RESULT

## Build image then run smoke test
test-image: build smoke-test

## Run all checks (unit tests + build + smoke test)
all: test test-image

## Install git pre-push hook to run smoke test before every push
install-hooks:
	@cp scripts/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "Pre-push hook installed. 'make build' before pushing to populate packet-browser:latest."
