# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from pathlib import Path

import pytest


REPO_ROOT = Path(__file__).resolve().parents[2]
GETTING_STARTED = REPO_ROOT / "docs" / "getting_started"
TUTORIALS = REPO_ROOT / "docs" / "tutorials"
HOW_TO = REPO_ROOT / "docs" / "how_to"

REMOVED_DOC_PYTHON_SURFACES = [
    GETTING_STARTED / "quickstart.py",
    GETTING_STARTED / "backtest_low_level.py",
    TUTORIALS / "backtest_fx_bars.py",
    GETTING_STARTED / "backtest_high_level.py",
    TUTORIALS / "backtest_orderbook_binance.py",
    TUTORIALS / "backtest_orderbook_bybit.py",
    HOW_TO / "loading_external_data.py",
    HOW_TO / "data_catalog_databento.py",
]


def _tutorial_id(path: Path) -> str:
    return f"{path.parent.name}/{path.name}"


@pytest.mark.parametrize("tutorial", REMOVED_DOC_PYTHON_SURFACES, ids=_tutorial_id)
def test_removed_doc_python_surface_is_absent(tutorial: Path) -> None:
    assert not tutorial.exists(), f"removed Python documentation script still exists: {tutorial}"


def test_docs_tree_has_no_python_scripts() -> None:
    assert list((REPO_ROOT / "docs").rglob("*.py")) == []
