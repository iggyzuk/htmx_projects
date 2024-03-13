inf-scroll:
	cargo watch -x 'run --bin htmx_infinite_scroll'

wordle:
	cargo watch -x 'run --bin htmx_wordle'

crud:
	cargo watch -x 'run --bin htmx_crud'

router:
	cargo watch -x 'run --bin htmx_router'

deploy:
	fly deploy