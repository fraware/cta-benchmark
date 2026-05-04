.PHONY: hf-package hf-upload hf-croissant hf-validate hf-scan-secrets hf-release

hf-package:
	python scripts/package_hf_dataset.py

hf-upload:
	python scripts/upload_hf_dataset.py

hf-croissant:
	python scripts/download_hf_croissant.py
	python scripts/add_rai_to_croissant.py

hf-scan-secrets:
	python scripts/scan_hf_release_secrets.py

hf-validate:
	python scripts/validate_neurips_artifact.py

hf-release: hf-package hf-upload hf-croissant hf-validate
