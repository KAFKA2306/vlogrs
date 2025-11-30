from pathlib import Path

import torch
from diffusers import DiffusionPipeline

from src.infrastructure.settings import settings


class ImageGenerator:
    def __init__(self):
        self._pipe = None

    def generate_from_novel(self, chapter_text: str, output_path: Path) -> None:
        prompt, negative_prompt = self._extract_prompt(chapter_text)
        self.generate(prompt, negative_prompt, output_path)

    def _extract_prompt(self, chapter_text: str) -> tuple[str, str]:
        # Generate optimized prompt using Jules
        from src.infrastructure.jules import JulesClient

        jules = JulesClient()
        text = jules.generate_image_prompt(chapter_text)

        # Hard filter to remove unwanted keywords
        import re

        text = re.sub(
            r"\b(pig|swine|hog|boar|piglet)s?\b", "", text, flags=re.IGNORECASE
        )
        text = re.sub(
            r"\b(translucent|transparent|semi-transparent|ethereal)\b",
            "",
            text,
            flags=re.IGNORECASE,
        )

        # Read prompts
        base_path = Path(__file__).parent
        template = (
            (base_path / "image_generator_prompt.txt")
            .read_text(encoding="utf-8")
            .strip()
        )
        negative_prompt = (
            (base_path / "image_generator_negative_prompt.txt")
            .read_text(encoding="utf-8")
            .strip()
        )

        return template.format(text=text), negative_prompt

    def generate(
        self, prompt: str, negative_prompt: str, output_path: Path, seed: int = None
    ) -> None:
        if not self._pipe:
            self._pipe = DiffusionPipeline.from_pretrained(
                settings.image_model,
                torch_dtype=torch.bfloat16,
                use_safetensors=True,
                device_map="balanced",
            )

        if seed is None:
            import random

            seed = random.randint(0, 2**32 - 1)

        print(f"Generating image with seed: {seed}")

        generator = torch.Generator(settings.image_device).manual_seed(seed)

        # Save prompt
        prompt_path = settings.photo_prompt_dir / f"{output_path.stem}.txt"
        prompt_path.parent.mkdir(parents=True, exist_ok=True)
        prompt_path.write_text(
            f"Prompt:\n{prompt}\n\nNegative Prompt:\n{negative_prompt}",
            encoding="utf-8",
        )

        image = self._pipe(
            prompt=prompt,
            negative_prompt=negative_prompt,
            height=settings.image_height,
            width=settings.image_width,
            num_inference_steps=settings.image_num_inference_steps,
            guidance_scale=settings.image_guidance_scale,
            generator=generator,
        ).images[0]

        image.save(output_path)
