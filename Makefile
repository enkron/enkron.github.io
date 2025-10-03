out_files = pub download index.html cv.html

.PHONY: site
site:
	@cargo r --release && python3 -m http.server 8080

.PHONY: clean
clean:
	@echo cleaning working tree\\n && \
		rm -rf $(out_files) && \
		for f in $(out_files); do \
			echo removing $$f; \
		done
