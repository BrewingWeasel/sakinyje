import type { Deck } from "@/components/settings/Deck.vue";

export interface ExportDetails {
	word: string;
	deck: string;
	model: string;
	sentence: string;
	defs: string[];
	fields: { [key: string]: string };
}

export interface Settings {
	dark_mode: boolean;
	languages: { [key: string]: LanguageSettings };
}

export interface LanguageSettings {
	deck: string;
	note_type: string;
	note_fields: { [key: string]: string };
	model: string;
	anki_parser: Deck[];
	dicts: [string, Dictionary][];
	frequency_list: string;
	words_known_by_freq: number;
}

export interface Word {
	text: string;
	lemma: string;
	morph: { [key: string]: string };
	clickable: boolean;
	rating: number;
}

export interface FileType {
	t: "TextSplitAt" | "StarDict";
	c: string | null;
}

export enum DictionaryType {
	File = "File",
	Url = "Url",
	Command = "Command",
	Wiktionary = "Wiktionary",
	EkalbaBendrines = "EkalbaBendrines",
	EkalbaDabartines = "EkalbaDabartines",
}

export interface Dictionary {
	t: DictionaryType;
	c: [string, FileType] | [string, string] | string | undefined;
}
