.PHONY: develop test stubs bench clean

develop:
	maturin develop --release

test: develop
	pytest -v

stubs: develop
	stubgen -m finance_query._finance_query -o finance_query/
	@echo "Stubs regenerated. Hand-polish async signatures in _finance_query.pyi before committing."

bench: develop
	python benches/bench_ticker.py

clean:
	cargo clean
	rm -rf .pytest_cache .mypy_cache __pycache__ dist target
