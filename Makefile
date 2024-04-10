inf-scroll:
	cargo watch -x 'run --bin htmx_infinite_scroll' -w htmx_infinite_scroll/src

wordle:
	cargo watch -x 'run --bin htmx_wordle' -w htmx_wordle/src

crud:
	cargo watch -x 'run --bin htmx_crud' -w htmx_crud/src

router:
	cargo watch -x 'run --bin htmx_router' -w htmx_router/src

images:
	cargo watch -x 'run --bin htmx_images' -w htmx_images/src

images-r:
	cargo watch -x 'run --bin htmx_images --release' -w htmx_images/src

auth:
	cargo watch -x 'run --bin htmx_auth' -w htmx_auth/src

email:
	cargo watch -x 'run --bin htmx_email' -w htmx_email/src

search:
	cargo watch -x 'run --bin htmx_search' -w htmx_search/src

events:
	cargo watch -x 'run --bin htmx_events' -w htmx_events/src

deploy:
	fly deploy