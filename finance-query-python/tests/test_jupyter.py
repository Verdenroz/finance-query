"""Smoke test for the Jupyter quickstart notebook."""

import json
from pathlib import Path


def test_quickstart_notebook_is_valid_json():
    nb_path = Path(__file__).parent.parent / "docs" / "examples" / "quickstart.ipynb"
    assert nb_path.exists(), f"notebook missing: {nb_path}"
    with nb_path.open() as f:
        data = json.load(f)
    assert data["nbformat"] == 4
    assert len(data["cells"]) > 0
    # Ensure at least one code cell exists
    code_cells = [c for c in data["cells"] if c["cell_type"] == "code"]
    assert len(code_cells) > 0


def test_quickstart_imports_finance_query():
    """The notebook should reference finance_query in its code cells."""
    nb_path = Path(__file__).parent.parent / "docs" / "examples" / "quickstart.ipynb"
    with nb_path.open() as f:
        data = json.load(f)
    code_text = "".join(
        "".join(c["source"]) for c in data["cells"] if c["cell_type"] == "code"
    )
    assert "finance_query" in code_text or "from finance_query" in code_text
