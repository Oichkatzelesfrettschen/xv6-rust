# Configuration file for the Sphinx documentation builder.
import os

#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "xv6-rust"
copyright = "2025, xv6 contributors"
author = "xv6 contributors"
version = "0.1"
release = "0.1"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.viewcode",
    "breathe",
]

templates_path = ["_templates"]
exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "sphinx_rtd_theme"

# Breathe configuration to include Doxygen output generated in docs/doxygen/xml
breathe_projects = {
    "xv6-rust": os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", "doxygen", "xml")
    )
}
breathe_default_project = "xv6-rust"
html_static_path = ["_static"]
