# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'xv6-rust'
copyright = '2025, xv6 contributors'
author = 'xv6 contributors'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = ['breathe']

templates_path = ['_templates']
exclude_patterns = []



# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'

# Breathe configuration to link Doxygen-generated XML
breathe_projects = {
    'xv6-rust': '../doxygen/xml'
}
breathe_default_project = 'xv6-rust'
html_static_path = ['_static']
