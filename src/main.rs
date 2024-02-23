use leptos::*;

pub mod formatter;

const EXAMPLE_INPUT: &str = r#"
/// Sets up the visual of an NPC
///
/// @param npc NPC to be affected
/// @param body_mesh mesh to be used as the body e.g. `HUM_BODY_NAKED0`
/// @param body_tex body texture assigned to this body mesh
/// @param skin body texture variant
/// @param head_mesh head mesh
/// @param head_tex head texture
/// @param teeth_tex teeth texture
/// @param armor_inst armor (C_ITEM instance) to be equipped or `-1` for no armor
func void Mdl_SetVisualBody( var instance npc,
                            var string body_mesh,
                            var int body_tex,
                            var int skin,
                            var string head_mesh,
                            var int head_tex,
                            var int teeth_tex,
                            var int armor_inst ) {};


/// Display the document using the document manager ID
///
/// @param docID document manager ID
func void Doc_Show(var int docID) {};


/// Create a new instance of the document manager with the arrow showing players position on the map and returns its ID.
///
/// @return Returns the ID of the document manager instance.
func int Doc_CreateMap() {};
"#;

fn main() {
    mount_to_body(|| view! {
        <h1>Daedalus docu comment formatter</h1>
        <p>Yes it is not perfect, but it sort of work, ok?</p>
        <App/>
    })
}

#[component]
fn App() -> impl IntoView {
    let (input, _set_input) = create_signal(EXAMPLE_INPUT.to_string());
    let (out, set_out) = create_signal("".to_string());

    view! {
        <textarea
            prop:value=move || input()
            on:input = move |ev| {
                _set_input(event_target_value(&ev))
            }
            rows="40" cols="50"
        >
            {move || input.get_untracked()}
        </textarea>

        <textarea prop:value=out rows="40" cols="50">
            {move || out}
        </textarea>

        <button
            on:click=move |_| {
                set_out(formatter::parse(&input()).expect("brh"));
            }
        >
            "Convert to MD"
        </button>
    }
}
