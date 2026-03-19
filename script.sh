find . -type f \( -name "*.ts" -o -name "*.tsx" \) \
  ! -path "*/node_modules/*" \
  ! -path "*/.next/*" | while read file; do
  echo "=== $file ===" >> codebase.txt
  cat "$file" >> codebase.txt
  echo "" >> codebase.txt
done
