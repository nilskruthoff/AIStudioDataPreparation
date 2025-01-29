# AI-Studio Data Preparation

The purpose of this repo is to test the extraction of data from various file types in preparation for
text and img embeddings for [AI-Studio](https://github.com/MindWorkAI/AI-Studio) in an isolated environment.

## CLI
However, for testing purposes it also serves as a CLI tool to transform data into **markdown** whenever possible:

```
Usage: AIStudioDataPreparation.exe [OPTIONS]

Options:
-p <PATH>, --path <PATH>    The path to the file containing the data to be extracted.
-h, --help                  Print help information.

Examples:
AIStudioDataPreparation.exe --path "/absolute/path/to/your/file"
AIStudioDataPreparation.exe -p "/absolute/path/to/your/file"
```

## Pdfium

Extracting pdf content relies on **Pdfium**, so you will need to have Pdfium available on your local machine.
The easiest way to set this up is to download the latest binaries from projects such as [bblanchon
/ pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) and place the `pdfium.dll' next to the rust executable.
This would be `target/release/' or `target/debug/'.
\
\
More information about binding the library can be found 
[here](https://github.com/ajrcarey/pdfium-render?tab=readme-ov-file#binding-to-pdfium)