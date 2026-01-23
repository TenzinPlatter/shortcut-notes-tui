NOTES_DIR=${NOTES_DIR:=/home/tenzin/gr/notes}

if [[ -n "$1" ]]; then
  NOTES_DIR="$1"
fi

if [[ ! -d "$NOTES_DIR" ]]; then
  echo "'$NOTES_DIR' is not a directory" >&2
  exit 1
fi

echo y | rm -rf $NOTES_DIR/*
