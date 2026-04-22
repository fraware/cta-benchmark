import Lake
open Lake DSL

package cta where
  leanOptions := #[
    ⟨`pp.unicode.fun, true⟩
  ]

@[default_target]
lean_lib CTA where
  roots := #[`CTA]
