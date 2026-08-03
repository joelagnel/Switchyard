# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

"""Guard that ``switchyard.__version__`` tracks the installed distribution."""

from importlib.metadata import version


def test_dunder_version_matches_installed_metadata() -> None:
    """``switchyard.__version__`` equals the installed distribution version.

    Guards against re-introducing a hardcoded ``__version__`` that could drift
    from pyproject.toml, the single source of truth.
    """
    import switchyard

    assert switchyard.__version__ == version("nemo-switchyard")
