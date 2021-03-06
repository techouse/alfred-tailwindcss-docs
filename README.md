# Tailwind CSS Docs Workflow for Alfred

![GitHub release](https://img.shields.io/github/release/techouse/alfred-tailwindcss-docs.svg)
![GitHub All Releases](https://img.shields.io/github/downloads/techouse/alfred-tailwindcss-docs/total.svg)
![GitHub](https://img.shields.io/github/license/techouse/alfred-tailwindcss-docs.svg)


Search the [Tailwind CSS documentation](https://tailwindcss.com/docs/) using [Alfred](https://www.alfredapp.com/).

![demo](demo.gif)

## Installation

1. [Download the latest version](https://github.com/techouse/alfred-tailwindcss-docs/releases/latest)
2. Install the workflow by double-clicking the `.alfredworkflow` file
3. You can add the workflow to a category, then click "Import" to finish importing. You'll now see the workflow listed in the left sidebar of your Workflows preferences pane.

## Usage

Just type `twd` followed by your search query.

```
twd background color
```

Either press `⌘Y` to Quick Look the result, or press `<enter>` to open it in your web browser.

## Changing Branches

The workflow supports searching the documentation of both branches, `v0`, `v1` and `v2`.
By default, it searches the `v2` branch. To search branch `v1` simply type `v1` **anywhere** in your query, like so:

```
twd padding v1
```

### Note

Built using [Alfred-Workflow](https://github.com/deanishe/alfred-workflow).
The lightning fast search is powered by [Algolia](https://www.algolia.com) using the _same_ index as the official [Tailwind CSS](https://tailwindcss.com/) website.
