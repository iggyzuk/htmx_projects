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

deploy:
	fly deploy