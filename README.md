# AI-Studio Data Preparation

The purpose of this repo is to test the extraction of data from various file types in preparation for
text and img embeddings for [AI-Studio](https://github.com/MindWorkAI/AI-Studio) in an isolated environment.

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
