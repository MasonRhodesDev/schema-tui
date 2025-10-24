#!/bin/bash

MODEL_TYPE="$1"
LANGUAGE="$2"

case "$LANGUAGE" in
  "en")
    if [ "$MODEL_TYPE" = "preview" ]; then
      echo '["whisper-en-tiny", "whisper-en-base", "whisper-en-small"]'
    else
      echo '["whisper-en-medium", "whisper-en-large", "whisper-en-large-v2"]'
    fi
    ;;
  "es")
    if [ "$MODEL_TYPE" = "preview" ]; then
      echo '["whisper-es-tiny", "whisper-es-base"]'
    else
      echo '["whisper-es-medium", "whisper-es-large"]'
    fi
    ;;
  "fr")
    if [ "$MODEL_TYPE" = "preview" ]; then
      echo '["whisper-fr-tiny", "whisper-fr-base"]'
    else
      echo '["whisper-fr-medium", "whisper-fr-large"]'
    fi
    ;;
  *)
    echo '[]'
    ;;
esac
