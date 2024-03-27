//! Generates the bindings for pxros.
//!
//! This is an implementation detail where both `Veecle` and `HighTec` are
//! working.
//! 
//! SPDX-FileCopyrightText: Veecle GmbH, HighTec EDV-Systeme GmbH
//! 
//! SPDX-License-Identifier: Apache-2.0
//! 


mod documentation_generator;
use std::fs;
use std::path::PathBuf;

use bindgen::callbacks::{ItemInfo, ParseCallbacks};
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use serde_json::Result;
use syn::{token, Block, FnArg, ForeignItem, Item, ItemFn, Pat, PatIdent, Signature, Visibility};
use pxros_hr;

use crate::documentation_generator::api_docs_generator::generate_comments;

fn main() {
    let outdir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let output_file = outdir.join("bindings.rs");

    // PXROS functions for which a safe wrapper should be generated.

    let safe_error_functions = &[SafeFunctionWrapper {
        function_name: "PxGetError",
        safety_reasoning: &["* Takes no parameters.", "* Returns safe [`PxError_t`]."],
    }];

    let safe_task_functions = &[SafeFunctionWrapper {
        function_name: "PxGetId",
        safety_reasoning: &["* Takes no parameters.", "* Returns safe [`PxTask_t`]."],
    }];

    let safe_message_functions = &[
        SafeFunctionWrapper {
            function_name: "PxMsgAwaitRel",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgAwaitRel_EvWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsgEvent_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgAwaitRel_NoWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetBuffersize",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns primitive [`u32`] wrapped in [`PxSize_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetMetadata",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns primitive [`u64`] wrapped in [`PxMsgMetadata_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetOwner",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxTask_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetProtection",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxProtectType_t`].",
                "* On error returns [`PxProtectType_t::NoAccessProtection`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetSender",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgGetSize",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns primitive [`u32`] wrapped in [`PxSize_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgInstallRelmbx",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxError_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgReceive",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgReceive_EvWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsgEvent_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgReceive_NoWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsgEvent_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgRequest",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgRequest_EvWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsgEvent_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgRequest_NoWait",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgSend",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgSend_Prio",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxMsg_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgSetMetadata",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxError_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgSetProtection",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxError_t`].",
            ],
        },
        SafeFunctionWrapper {
            function_name: "PxMsgSetToAwaitRel",
            safety_reasoning: &[
                "* Parameters are copied and checked by PXROS.",
                "* Returns safe [`PxError_t`].",
            ],
        },
    ];

    // Concatenate all safe function collections.
    let safe_functions = [
        safe_error_functions.as_ref(),
        safe_task_functions.as_ref(),
        safe_message_functions.as_ref(),
    ]
    .concat();

    let bindings = bindgen::Builder
        ::default()
        .header_contents("wrapper.h", pxros_hr::WRAPPER)
        .use_core()
        // Allows us to use well-sized types for primitives, see module documentation.
        .ctypes_prefix("crate::bindings::ffi")
        // FIXME: It has not been verified yet whether all enumerations are safe
        // to be used as rust enumerations, the bindings might contain UB!
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .default_alias_style(bindgen::AliasVariation::NewType)
        // The following types add no semantic, so we make them transparent type exports.
        .type_alias("PxSize_t")
        .type_alias("PxUInt_t")
        .type_alias("PxULong_t")
        .type_alias("PxLong_t")
        // Just an alias for `_PxProtectRegion_T`.
        .type_alias("PxProtectRegion_T")
        // The following blocklisted types are manually defined, see src/bindings.rs
        .blocklist_item("PxOpool_t")
        .blocklist_item("PxMc_t")
        .blocklist_item("PxTask_t")
        .blocklist_item("PxPe_t")
        .blocklist_item("PxMbx_t")
        .blocklist_item("PxMsg_t")
        .blocklist_item("PxTask_t")
        .sort_semantically(true)
        // The tests are meant to be run on the architecture for which they are
        // compiled. We do not want to run them on the host, and at the moment
        // we cannot run them on a tricore chip.
        .layout_tests(false)
        .prepend_enum_name(false)
        // The default callbacks will emit `cargo:rerun-if-changed=_` for each header file.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(
            Box::new(
                DeriveDefmtCallbacks::new(
                    &[
                        "PxError_t",
                        "PxMessageClass_t",
                        "PxSvc_t",
                        "PxProtectType_t",
                        "_PxObjType_t",
                        "PxOpoolType_t",
                        "PxMcType_t",
                        "_PxTmodebits",
                        "PxMsgType_t",
                        "PxStackSpecType_t",
                        "PxSchedExtCause_t",
                        "PxSchedCause_t",
                        "PxTaskStackTypes_t",
                        "PxTraceCtrl_t",
                        "PxMbxReq_t",
                        "PxIntSvEnum_t",
                        "PxIntType_t",
                    ]
                )
            )
        )
        .parse_callbacks(Box::new(PrependUnderscoresCallback::new(&safe_functions)))
        // Bindgen cannot see the Hightec toolchain, so we need to configure a
        // similar target here manually.
        .clang_args(["-target", "i386"])
        .generate()
        .expect("Unable to generate bindings");

    let bindings_with_safe_wrappers = generate_safe_function_wrappers(bindings.to_string(), &safe_functions);
    let bindings_with_safe_wrappers_and_docs = inject_pxapi_doc(bindings_with_safe_wrappers, &safe_functions)
        .expect("Failed to inject PXROS API documentation!");

    fs::write(output_file.clone(), bindings_with_safe_wrappers_and_docs).expect("Couldn't write bindings!");

    // Format the resulting bindings. Requires rustfmt to be installed.
    if !std::process::Command::new("rustfmt")
        .args([output_file.to_str().unwrap()])
        .status()
        .unwrap()
        .success()
    {
        panic!("Rustfmt encountered an error.");
    }
}

/// Allows to specify the name of types for which `defmt::Format` should be
/// derived.
#[derive(Debug)]
struct DeriveDefmtCallbacks {
    derive_defmt: Vec<String>,
}

impl DeriveDefmtCallbacks {
    fn new(derive_defmt: &[&str]) -> Self {
        Self {
            derive_defmt: derive_defmt.iter().map(|type_name| type_name.to_string()).collect(),
        }
    }
}

impl ParseCallbacks for DeriveDefmtCallbacks {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if self.derive_defmt.contains(&info.name.to_owned()) {
            vec!["defmt::Format".to_owned()]
        } else {
            vec![]
        }
    }
}

/// Adds "__" to every function name.
#[derive(Debug)]
struct PrependUnderscoresCallback {
    function_names: Vec<&'static str>,
}

impl PrependUnderscoresCallback {
    fn new(safe_functions: &[SafeFunctionWrapper]) -> Self {
        Self {
            function_names: safe_functions
                .iter()
                .map(|safe_function| safe_function.function_name)
                .collect(),
        }
    }
}

impl ParseCallbacks for PrependUnderscoresCallback {
    fn generated_name_override(&self, item_info: ItemInfo<'_>) -> Option<String> {
        if self.function_names.contains(&item_info.name) {
            let mut item_name = "__".to_owned();
            item_name.push_str(item_info.name);
            Some(item_name)
        } else {
            None
        }
    }
}

/// Combines function name and safety reasoning to enforce safety reasoning for every type that is wrapped.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct SafeFunctionWrapper {
    function_name: &'static str,
    safety_reasoning: &'static [&'static str],
}

/// Creates safe wrappers around unsafe PXROS functions.
///
/// Some PXROS functions are safe to call even though they are FFI.
/// This function wraps them in a safe function to reduce boilerplate code.
fn generate_safe_function_wrappers(bindings: String, safe_functions: &[SafeFunctionWrapper]) -> String {
    let file = syn::parse_file(bindings.as_str()).unwrap();

    let generated_functions: Vec<ItemFn> = file
        .items
        .iter()
        .filter_map(|item| try_generate_safe_function_wrapper(item, safe_functions))
        .collect();

    format!("{}", quote! {
        #file
        #(#generated_functions)*
    })
}

/// Creates a wrapper for the supplied [`Item`] if it is contained within the `safe_functions`.
///
/// Returns `None` if the item is not part of `safe_functions`.
fn try_generate_safe_function_wrapper(item: &Item, safe_functions: &[SafeFunctionWrapper]) -> Option<ItemFn> {
    // Filter anything but foreign functions with a name matching a safe function.
    let Item::ForeignMod(item) = item else {
        return None;
    };
    let Some(foreign_item) = item.items.get(0).cloned() else {
        return None;
    };
    let ForeignItem::Fn(foreign_function) = foreign_item else {
        return None;
    };
    let foreign_function_name = foreign_function.sig.ident.to_string();

    // To compare with the `safe_function.function_name` the prefix added by `PrependUnderscoresCallback` needs to
    // be removed.
    let Some(stripped_function_name) = foreign_function_name.as_str().strip_prefix("__") else {
        return None;
    };

    if !safe_functions
        .iter()
        .any(|safe_function| safe_function.function_name == stripped_function_name)
    {
        return None;
    }

    let underscored_ident = foreign_function.sig.ident.clone();
    let non_underscored_ident = underscored_ident.to_string();
    let non_underscored_ident = non_underscored_ident.strip_prefix("__").unwrap();
    let non_underscored_ident = syn::Ident::new(non_underscored_ident, underscored_ident.span());

    let function_arguments: Vec<PatIdent> = foreign_function
        .sig
        .inputs
        .iter()
        .filter_map(|input| {
            let FnArg::Typed(input) = input else {
                return None;
            };
            let Pat::Ident(input) = *input.pat.clone() else {
                return None;
            };
            Some(input)
        })
        .collect();

    let unsafe_function_call_block: Vec<syn::Stmt> =
        syn::parse_quote! { unsafe { #underscored_ident(#(#function_arguments),*) } };
    let wrapper_function = ItemFn {
        attrs: vec![],
        vis: Visibility::Public(token::Pub {
            // Span does not matter for us, the information is only used in macros.
            span: Span::call_site(),
        }),
        sig: Signature {
            ident: non_underscored_ident,
            ..foreign_function.sig
        },
        block: Box::new(Block {
            brace_token: Default::default(),
            stmts: unsafe_function_call_block,
        }),
    };
    Some(wrapper_function)
}

/// Injects PXROS API and Veecle safety docs into the generated bindings.
///
/// It may panic if the used regex formula is incorrect or the constructed file path is illegal.
///
/// The regex matches "pub fn Px_function_name(", including optional white space characters among the tokens.
///
/// The first capture group contains actual function name.
fn inject_pxapi_doc(bindings: String, safe_functions: &[SafeFunctionWrapper]) -> Result<String> {
    let mut out_bindings = bindings.clone();
    let re = Regex::new(r"pub\s+fn\s+(Px[a-zA-Z0-9_]*)\s*\(").unwrap();

    for caps in re.captures_iter(&bindings) {
        if let Some(matched_function_name) = caps.get(1) {
            let mut api_doc_path = PathBuf::from("./pxros-hr/api-src/");
            api_doc_path.push(matched_function_name.as_str().to_owned() + ".json");
            // Assume missing API file is ok (there is no matching JSON for all the Px... funcs).
            if api_doc_path.exists() {
                let api_doc_path = api_doc_path.to_str().unwrap();
                println!("PXDOCGEN: Processing: {}", api_doc_path);
                let mut apidoc = generate_comments(api_doc_path);

                // Add safety docs to apidocs.
                if let Some(safe_function) = safe_functions
                    .iter()
                    .find(|safe_function| safe_function.function_name == matched_function_name.as_str())
                {
                    apidoc.push_str("///\n");
                    apidoc.push_str("/// ### Safety reasoning (Veecle):\n");

                    for safety_reasoning_line in safe_function.safety_reasoning {
                        apidoc.push_str("/// ");
                        apidoc.push_str(safety_reasoning_line);
                        apidoc.push('\n');
                    }
                }

                let matched_function = caps.get(0).unwrap().as_str();
                apidoc.push_str(matched_function);

                out_bindings = out_bindings.replace(matched_function, apidoc.as_str());
            }
        }
    }

    Ok(out_bindings)
}
