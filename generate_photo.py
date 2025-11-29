import argparse
import logging
import sys
from pathlib import Path

from dotenv import load_dotenv

# Add project root to path
sys.path.append(str(Path(__file__).parent))

from src.infrastructure.image_generator import ImageGenerator


def main():
    load_dotenv()
    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger(__name__)

    parser = argparse.ArgumentParser(description="Generate photo from novel chapter")
    parser.add_argument("novel_path", type=Path, help="Path to the novel markdown file")
    args = parser.parse_args()

    novel_path = args.novel_path
    if not novel_path.exists():
        logger.error(f"File not found: {novel_path}")
        sys.exit(1)

    # Determine output path
    # Assuming novel path is data/novels/YYYYMMDD.md
    # Output should be data/photos/YYYYMMDD.png
    photos_dir = Path("data/photos")
    photos_dir.mkdir(parents=True, exist_ok=True)
    
    output_filename = novel_path.stem + ".png"
    output_path = photos_dir / output_filename

    logger.info(f"Reading novel from: {novel_path}")
    chapter_text = novel_path.read_text(encoding="utf-8")

    logger.info("Initializing ImageGenerator...")
    generator = ImageGenerator()

    logger.info(f"Generating image to: {output_path}")
    generator.generate_from_novel(chapter_text, output_path)
    
    logger.info("Done!")

if __name__ == "__main__":
    main()
