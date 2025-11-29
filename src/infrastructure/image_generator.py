from pathlib import Path

import torch
from diffusers import ZImagePipeline

from src.infrastructure.settings import settings


class ImageGenerator:
    def __init__(self):
        self._pipe = None

    def generate_from_novel(self, chapter_text: str, output_path: Path) -> None:
        prompt = self._extract_prompt(chapter_text)
        self.generate(prompt, output_path)

    def _extract_prompt(self, chapter_text: str) -> str:
        lines = [line.strip() for line in chapter_text.split("\n") if line.strip()]

        # Extract first few meaningful paragraphs (skip headings)
        paragraphs = [
            line for line in lines if len(line) > 20 and not line.startswith("#")
        ][:3]

        if not paragraphs:
            return "VRChat scene, anime style, detailed background, cinematic lighting"

        # Combine paragraphs for richer context
        combined = " ".join(paragraphs)[:500]

        # Create prompt emphasizing visual atmosphere and VRChat aesthetic
        prompt = (
            f"Scene from a VRChat virtual world story: {combined}. "
            "Anime-style illustration, vibrant colors, detailed environment, "
            "cinematic composition, soft lighting, 8k resolution"
        )

        return prompt

    def generate(self, prompt: str, output_path: Path) -> None:
        if not self._pipe:
            self._pipe = ZImagePipeline.from_pretrained(
                settings.image_model,
                torch_dtype=torch.bfloat16,
                low_cpu_mem_usage=True,
                device_map="balanced",
            )

        image = self._pipe(
            prompt=prompt,
            height=settings.image_height,
            width=settings.image_width,
            num_inference_steps=settings.image_num_inference_steps,
            guidance_scale=settings.image_guidance_scale,
            generator=torch.Generator("cuda").manual_seed(settings.image_seed),
        ).images[0]

        image.save(output_path)
