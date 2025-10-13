SHELL := /bin/bash
out_files = pub priv download index.html cv.html 404.html

.PHONY: site
site:
	@wasm-pack build --target web --out-dir web/pkg && \
		cargo r --release && python3 -m http.server 8080

ENKRONIO_LOCK_KEY := "test"
.PHONY: test
test:
	@cargo r -q -- add "Test Entry" && \
	export ENKRONIO_LOCK_KEY=$(ENKRONIO_LOCK_KEY) \
	file=$$(ls in/entries/*-test-entry.md | head -n1) && \
	[ -n "$$file" ] && echo "##test>" >> "$$file" && \
	cargo r -q -- lock "$$file"

.PHONY: clean
clean:
	@echo cleaning working tree\\n && \
	rm -rf $(out_files) && \
	for f in $(out_files); do \
		echo Removing $$f; \
	done
	@git clean -f && git restore .
