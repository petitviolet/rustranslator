# rustranslator

Call Google Translation v3(Advanced) API from Rust

## Usage

```console
$ cargo build
$ export GOOGLE_APPLICATION_CREDENTIALS=/path/to/credential
$ GOOGLE_ACCESS_TOKEN=$(gcloud auth application-default print-access-token) ./target/debug/rustranslator 'こんにちは' en
Hello
```

## Google Translation v3 API

See the document at first.
https://cloud.google.com/translate/docs/advanced/translating-text-v3?hl=en#translating_input_strings
