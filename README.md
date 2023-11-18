# Sakinyje

Sakinyje is a tool for language learning based on the idea of [sentence mining](https://refold.la/roadmap/stage-2/a/basic-sentence-mining).

## Features:
- export sentence to Anki (using ankiconnect)
- automatically get definition from user supplied dictionaries
  - allows fetching definitions from multiple different dictionaries
  - allows multiple ways to get definition 
    - stardict
    - from a server
    - file with special delimiter
- Customizable, config saved to toml file in configuration directory
- Definitions and the auto generated lemma are editable
- NLP with SpaCy which allows for:
  - automatic lemma (base word) detection
  - coloring of words based on grammatical characteristics
  - easy support for the following languages:
    - Catalan
    - Chinese
    - Croatian
    - Danish
    - Dutch
    - English
    - Finnish
    - French
    - German
    - Greek
    - Italian
    - Japanese
    - Korean
    - Lithuanian
    - Macedonian
    - Norwegian
    - Polish
    - Portuguese
    - Romanian
    - Russian
    - Slovenian
    - Spanish
    - Swedish
    - Ukrainian
  - training custom models for other languages or improving the models of existing languages (see https://spacy.io/usage/training for training instructions and https://universaldependencies.org/ for source data)

## Future plans: 
- check the issues tab

## Installation

### Use prebuilt binary
You can find prebuilt binaries of the latest release (https://github.com/BrewingWeasel/sakinyje/releases)
Note that currently the AppImage does not work, and the others are not tested

### Build from source
Dependencies (debian based):
libgtk-3-dev libwebkit2gtk-4.0-dev libayatana-appindicator3-dev librsvg2-dev glibc-source libc6 python3-dev

#### Using Just
```sh
git clone https://github.com/BrewingWeasel/sakinyje && cd sakinyje
just build
```
You can also manually build using cargo-tauri with trunk and spacy.

In either case, your final build will be in target/release/sakinyje, and the bundles will be in target/release/bundles

## Usage (WIP):
### First time setup:
Add a spacy model (either download one from https://spacy.io/models or train one)
Add a dictionary. Here are some suggestions:
- If you're learning Lithuanian I have a good server based one that uses dabartinės kalbos žodynas prebuilt: https://github.com/BrewingWeasel/lithdict/blob/main/uuids
- https://freedict.org/downloads/ has some decent dictionaries for a fair amount of languages
- https://github.com/Vuizur/Wiktionary-Dictionaries has a ton of dictionaries based on old wiktionary data. Be sure to download the stardict version though.

### Normal usage:
Copy or write a sentence into the reader tab, then press parse.
Click on any word to get its definition.
Click on the export button to export to Anki

## Name
The name comes from the locative form of the Lithuanian word for sentence (sakinys).
