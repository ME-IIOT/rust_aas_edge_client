#!/bin/bash

# Define the directory to search in, replace "/path/to/your/project" with your actual project path
PROJECT_DIR="./backend"

# Authors' information
AUTHORS_INFO="// Authors: Manh-Linh Phan (manh.linh.phan@yacoub.de), Xuan-Thuy Dang (xuan.thuy.dang@yacoub.de), Markus Rentschler\n\n"

# Find all .rs files and prepend the authors' information
find "$PROJECT_DIR" -type f -name "*.rs" -exec sh -c 'echo "$1\n$(cat "$0")" > "$0"' {} "$AUTHORS_INFO" \;

