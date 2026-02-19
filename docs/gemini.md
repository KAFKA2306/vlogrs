# Gemini Model Lineup (2026 Snapshot)

As of early 2026, the Gemini family has matured into the Gemini 3 series, offering frontier-class performance across various specializations.

## Primary Models

| Model Name | ID (API) | Description |
| :--- | :--- | :--- |
| **Gemini 3 Flash** | `gemini-3-flash` | Released Dec 2025. Default high-speed model. Optimized for latency and cost. |
| **Gemini 3 Pro** | `gemini-3-pro` | Released Nov 2025. Most powerful reasoning and performance model. |
| **Gemini 3 Deep Think** | `gemini-3-deep-think` | Specialized version for complex scientific and engineering challenges. Updated Feb 2026. |

## Legacy / Fallback Models

| Model Name | ID (API) | Notes |
| :--- | :--- | :--- |
| **Gemini 1.5 Flash** | `gemini-1.5-flash` | High-efficiency legacy model. Stable fallback. |
| **Gemini 1.5 Pro** | `gemini-1.5-pro` | High-capacity legacy model. |
| **Gemini 2.0 Flash** | `gemini-2.0-flash` | Released Jan 2025. Previous default. |

---

## Configuration in VLog

VLog uses **Gemini 3 Flash** as its primary engine for transcription analysis and summary generation to balance speed and accuracy in its "Autonomous Life Logger" role.

> [!NOTE]
> If `gemini-3-flash` is not yet available in your specific regional API endpoint, use `gemini-1.5-flash` or `gemini-2.0-flash` as a fallback in `.env`.
