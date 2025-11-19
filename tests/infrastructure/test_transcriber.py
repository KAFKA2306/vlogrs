from unittest.mock import MagicMock, patch
from src.infrastructure.transcriber import Transcriber

def test_transcribe_lazy_loads_model():
    transcriber = Transcriber()
    
    with patch("src.infrastructure.transcriber.WhisperModel") as MockModel:
        mock_instance = MockModel.return_value
        mock_segment = MagicMock()
        mock_segment.text = "Hello world"
        mock_instance.transcribe.return_value = ([mock_segment], None)
        
        result = transcriber.transcribe("dummy.wav")
        
        assert result == "Hello world"
        MockModel.assert_called_once()
        mock_instance.transcribe.assert_called_once()
