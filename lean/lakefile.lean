import Lake
open Lake DSL

require "leanprover-community" / "mathlib" @ git "v4.12.0"

package cta where
  leanOptions := #[
    ⟨`pp.unicode.fun, true⟩
  ]

@[default_target]
lean_lib CTA where
  roots := #[`CTA]
