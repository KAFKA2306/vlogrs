import argparse
import os
import torch
from diffusers import DiffusionPipeline, FlowMatchEulerDiscreteScheduler
from PIL import Image

def main() -> None:
    parser: argparse.ArgumentParser = argparse.ArgumentParser()
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--negative-prompt", default="")
    args: argparse.Namespace = parser.parse_args()

    model_id: str = os.getenv("IMAGE_MODEL", "Tongyi-MAI/Z-Image-Turbo")
    
    pipe: DiffusionPipeline = DiffusionPipeline.from_pretrained(
        model_id,
        torch_dtype=torch.bfloat16,
        use_safetensors=True
    )
    pipe.scheduler = FlowMatchEulerDiscreteScheduler.from_config(pipe.scheduler.config)
    pipe.to("cuda")
    
    image: Image.Image = pipe(
        prompt=args.prompt,
        negative_prompt=args.negative_prompt,
        num_inference_steps=9,
        guidance_scale=0.0,
        width=1024,
        height=1024
    ).images[0]
    
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    image.save(args.output)

if __name__ == "__main__":
    main()
