# Dioxus translations

The desktop UI uses the `dioxus-i18n` 0.6 provider recommended by the
[Dioxus 0.7 guide](https://dioxuslabs.com/learn/0.7/guides/utilities/internationalization/).
The root initializes embedded Fluent resources once per window. The fallback is
English. General > Language changes the reactive provider and saves the existing
local `lang` option. New windows load that preference; existing separate remote
processes keep their current language until reopened.

The first locales are English, Simplified Chinese and Traditional Chinese.
System default recognizes Chinese scripts and Taiwan, Hong Kong and Macao;
other system languages use English until their resources are added. Existing
`src/lang` tables remain available to the core and supplied the established
Chinese wording. They are not changed by this UI integration.

Existing UI labels use `tr("Default Codec")`, exported by the UI prelude.
The adapter converts an English key to its Fluent ID (`default-codec`) and
subscribes the rendering component to the provider. IDs lowercase ASCII words
and join punctuation/space boundaries with hyphens. Keep these keys stable and
avoid conflicting IDs. Add the same message ID to each `.ftl` file. Fluent
resources are embedded with `include_str!`; installation does not need external
translation files. Missing messages fall back to English; unknown runtime
messages retain their original text.

Translate presentation text only. Peer names, chat messages, passwords, backend
names, codec names, option values and session commands keep their original data.
In particular, settings and sort choices retain their existing English/internal
values while translating the visible label. Do not use translated strings in
comparisons or configuration writes. Parameterized future messages can use
`dioxus_i18n::t!` / `try_translate_with_args` under the same provider.
