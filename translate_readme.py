import ollama
import os

# --- Configuration ---
INPUT_FILE = "README-Chinese.md"
OUTPUT_FILE = "README.md"
MODEL_NAME = "qwen3"

SYSTEM_PROMPT = (
    "You are a professional translator. Translate the following Markdown text into English. "
    "Crucially: preserve all HTML tags, Markdown syntax, code blocks, and links exactly as they are. "
    "Only translate the human-readable text content."
)

def translate_readme():
    if not os.path.exists(INPUT_FILE):
        print(f"Error: {INPUT_FILE} not found.")
        return

    with open(INPUT_FILE, "r", encoding="utf-8") as f:
        content = f.read()

    print(f"Translating {INPUT_FILE} using {MODEL_NAME}...")

    try:
        response = ollama.generate(
            model=MODEL_NAME,
            system=SYSTEM_PROMPT,
            prompt=f"Translate this text to English:\n\n{content}",
            stream=False,
            options={
                "temperature": 0.3,  # Lower temperature for more literal translation
            }
        )

        translated_text = response['response']

        with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
            f.write(translated_text)

        print(f"Success! Translated file saved as {OUTPUT_FILE}")

    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    translate_readme()
