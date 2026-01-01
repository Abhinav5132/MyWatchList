
use crate::{backend::sign_up::check_availability, frontend::{login_popup::Login, login_popup::LoginStruct,manage_user_profile::{ChangePfp, choose_image}}};
pub use crate::frontend::*;

pub enum Steps {
    WELCOME,
    APPTYPE,
    USEREGISTRATION, // configure sync server should always be before user registration. As we check if the user is registered on that specific sync server.
    CONFIGURESYNCSERVER, // only encountered if they choose PERSONALSYNC OR PUBLICSYNC
    CHOOSEUPDATESCHEDULE,
}

pub enum AccountType {
    LOGIN,
    REGISTER,
    LOCAL,
    TOBEDETERMINED,
}
pub enum AppType {
    LOCAL,
    PERSONALSYNC, // personal sync should guide you though installing and setting up your own sync server.
    PUBLICSYNC,// act the same way for now
}
#[derive(Clone)]
struct OnboardingState {
    app_type: Signal<AppType>,
    acc_type: Signal<AccountType>,
}

#[derive(Clone)]
enum UpdateScehdule{
    OnStartUp,
    OnceADay,
    OnceAWeek,
    OnceAMonth,
    Never
}

#[derive(PartialEq)]
pub enum LoginError{
    PasswordNotSame,
    EmailUnavailable,
    UsernameUnavailable,
    None
}

#[derive(Clone)]
struct LoginState{
    username: Signal<String>,
    password: Signal<String>,
    email: Signal<String>,
    password_again: Signal<String>,
    pfp: Signal<String>,
}

#[derive(Serialize)]
pub struct CheckAvailability {
    username: String,
    email: String,
}

#[derive(Deserialize)]
pub struct AvailabilityResponse {
    pub username_available: bool,
    pub email_available: bool,
}

#[derive(Serialize)]
pub struct SignUpStruct {
    user_name: String,
    user_password: String,
    user_email: String,
    user_pfp: Option<String>,
}

#[component]
pub fn FirstTimePage() -> Element{
    let mut step = use_signal(|| Steps::WELCOME);
    let mut app_type = use_signal(|| AppType::LOCAL);
    let mut acc_type = use_signal(|| AccountType::LOCAL);
    provide_context(OnboardingState {
        app_type,
        acc_type,
    });     
    rsx!(
        match *step.read(){
            Steps::WELCOME => rsx!(
                WelcomePage { 
                    on_next: move || step.set(Steps::APPTYPE),
                }
            ),
            Steps::APPTYPE => rsx!(
                SelectAppType { 
                    on_next: move || step.set(Steps::CONFIGURESYNCSERVER),
                    on_back: move || step.set(Steps::WELCOME)
                }
            ),

            Steps::USEREGISTRATION => {
                rsx!(
                    UserRegistrations { 
                        on_next: move || step.set(Steps::CHOOSEUPDATESCHEDULE),
                        on_back: move || step.set(Steps::CONFIGURESYNCSERVER)
                     }
                )
            },
            Steps::CONFIGURESYNCSERVER => {
                match *app_type.read(){
                    AppType::LOCAL => {
                        rsx!(
                            ConfigureSyncServerLocal { 
                                on_next: move || step.set(Steps::USEREGISTRATION),
                                on_back: move || step.set(Steps::APPTYPE)
                            }
                        )
                    },
                    AppType::PERSONALSYNC => {
                        rsx!(
                            ConfigureSyncServerPersonal { 
                                on_next: move || step.set(Steps::USEREGISTRATION),
                                on_back: move || step.set(Steps::APPTYPE)
                            }
                        )
                    },

                    AppType::PUBLICSYNC => {
                        rsx!(
                            ConfigureSyncServerPrivate { 
                                on_next: move || step.set(Steps::USEREGISTRATION),
                                on_back: move || step.set(Steps::APPTYPE)
                             }
                        )
                    },
                }
            },
            Steps::CHOOSEUPDATESCHEDULE => {
                rsx!(
                    div { 
                        "This shit is not done bro"
                     }
                )
            }
        }  
    )
}

#[component]
pub fn WelcomePage(on_next: EventHandler<()>) -> Element{
    rsx!(
        section { class: "hero",
            h1 { "Your watch list. Your rules." }
            p {
                "Create, organize, and share watch lists with friends — without tracking, ads, or data harvesting."
            }
            button {
                class: "primary_cta",
                onclick: move |_| on_next.call(()),
                "Get Started"
            }
        }

        section { 
                class: "features",
                FeatureCard {
                    title: "Create Watch Lists",
                    description: "Build custom watch lists for anime and share with friends."
                }
                FeatureCard {
                    title: "Control your data",
                    description: "Share lists with friends or keep them all to yourself. We give the choice back to you."
                }
                FeatureCard {
                    title: "Privacy First",
                    description: "No tracking. No analytics. No selling your data. Your watch habits stay yours."
                }
                FeatureCard {
                    title: "Host your own server",
                    description: "Host your own sync server with your friends allowing full control over your data."
                }
            }

            section { 
                class: "privacy",
                h2 { "We don't watch you watch." }
                p {
                    "Unlike most platforms, we don't track your behavior, sell your data, or build ad profiles. "
                    "Your watch history is yours alone."
                }
            }
    )
}

#[component]
fn FeatureCard(title: &'static str, description: &'static str) -> Element {
    rsx!(
        div { class: "feature_card",
            h3 { "{title}" }
            p { "{description}" }
        }
    )
}

#[component]
pub fn SelectAppType(on_next: EventHandler<()>, on_back: EventHandler<()>)->Element{
    let mut state = use_context::<OnboardingState>();
    rsx!(
        div { 
            class: "select-app-container",
            h3 { 
                class: "FirstTimeQuestion",
                "Select how you want to use MyWatchList"
            }   
            div {
                class: "Button_wrapapper",
                div { 
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.app_type.set(AppType::LOCAL); // themes and other stuff can be added here.
                        state.acc_type.set(AccountType::LOCAL);
                        on_next.call(());
                    },
                    "Use the app locally"
                }
                span { 
                    class: "selectAppTypeButtonToolTip",
                    "Local apps do not use a sync server to provide content updates or allow social features. 
                    Everything is stored on your device and the app can be used offline. 
                    Note: the app will need periodic internet access if you want all the latest shows and movies."
                }
            }
            div {
                class: "Button_wrapapper",
                div { 
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.app_type.set(AppType::PERSONALSYNC);
                        state.acc_type.set(AccountType::TOBEDETERMINED);
                        on_next.call(());
                    },
                    "Setup a personal sync server."
                }

                span { 
                    class: "selectAppTypeButtonToolTip",
                    "Set up a personal MyWatchList sync server allowing you and 
                    your friends to share recommendations and watch lists while 
                    also cuting down update times significantly. 
                    All the data is stored on the sync server which can only be acessed by you and your friends. 
                    Note: requires a seperate computer/server that can run 24/7. 
                    A raspberry pi will work if your only hosting for you and your friends."
                }
            }
            div {
                class: "Button_wrapapper",
                div { 
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.app_type.set(AppType::PUBLICSYNC);
                        state.acc_type.set(AccountType::TOBEDETERMINED);
                        on_next.call(());
                    },
                    "Connect to a public sync server"
                }

                span {
                    class: "selectAppTypeButtonToolTip",
                    "Use this option if your trying to connect to your friends server or 
                    use a publically available sync server. Caution all data is 
                    stored on the public sync server and the responsibility 
                    for your data is on the server owner."
                }
            }

            button { 
                class: "backButton",
                onclick: move |_| {
                    on_back.call(());
                },
                "Go back"
            }
        }
        

    )
}

#[component]
pub fn ConfigureSyncServerPrivate(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element{
    rsx!(

    )
}

#[component]
pub fn ConfigureSyncServerPersonal(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element{

    rsx!(
        
    )
}

#[component]
pub fn ConfigureSyncServerLocal(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element{
    rsx!(
        div { 
            class:"ConfigureSyncServer",
            p { 
                id:"ConfigureLocal",
                "Using the local app dosent require setting up the sync server. 
                If you change your mind you can set up a sync server anytime in the settings.
                You can continue safely OR you can stay here I guess.
                "
            }

            button { 
                class:"continueButton",
                onclick: move |_| {
                    on_next.call(());
                },
                "This page is redundant take me to the next"
            }
            button { 
                class:"continueButton",
                onclick: move |_| {
                    on_back.call(());
                },
                "Take me back I regret this"
            }
        }
    )
}

#[component]
pub fn UserRegistrations(on_next: EventHandler<()>, on_back: EventHandler<()>)-> Element{
    let mut state = use_context::<OnboardingState>();
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut password_again = use_signal(|| "".to_string());
    let mut pfp = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());

    provide_context(LoginState{
        username,
        password,
        email,
        password_again,
        pfp
    });
    rsx!(
        match *state.acc_type.read() {
            AccountType::LOCAL =>{
                rsx!( 
                    FullRegistration { 
                        on_next: move || on_next.call(()),
                        on_back: move || on_back.call(())
                    }
                )
            }

            AccountType::LOGIN => {
                rsx!(
                    LoginRegistrations { 
                        on_next: move || on_next.call(()),
                        on_back: move || on_back.call(())
                     }
                )
            }

            AccountType::REGISTER => {
                rsx!( 
                    FullRegistration { 
                        on_next: move || on_next.call(()),
                        on_back: move || on_back.call(())
                    }
                )
            }

            AccountType::TOBEDETERMINED => {
                rsx!(
                    div { class: "selectLoginTypeContainer",
                        div {
                            class: "selectLoginTypeButton",
                            onclick: move |_| {
                                state.acc_type.set(AccountType::LOGIN);
                            },
                            "Login"
                        }
                        div {
                            class: "selectLoginTypeButton",
                            onclick: move |_| {
                                state.acc_type.set(AccountType::REGISTER);
                            },
                            "Register"
                        }

                        div { 
                            class: "backButton",
                            onclick: move |_| {
                                on_back.call(());
                            },
                            "Go back"
                        }
                    }
                )
            }
        }
    )
}

#[component]
pub fn LoginRegistrations(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element{
    let mut loginState = use_context::<LoginState>();
    rsx!(
        div { 
            class:"userRegistrationsContainer",
            div { 
                class: "UserFieldsContainer",
                label { "Username:" },
                input{
                    id: "UserNameInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.username.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('EmailInput').focus();"#);
                    }
                }

                label { "Password:" },
                input{
                    id: "PasswordInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.password.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('PasswordAgainInput').focus();"#);
                    }
                }
            }

            div { id: "ButtonsContainer",
                button { class: "submitButton",
                    onclick: move |_| {
                        spawn(async move {
                            let client = Client::new();
                            let username = loginState.username.read().to_string();
                            let password = loginState.password.read().to_string();
                            if let Ok(res) = client.post("http://localhost:3000/login").json(
                                &LoginStruct{
                                    username,
                                    password
                                }
                            ).send().await && res.status().is_success(){
                                on_next.call(());
                            }
                        });
                    }
                }
                button { 
                    class: "backButton",
                    onclick: move |_| {
                        on_back.call(());
                    },
                    "Go back"
                }
            }
        }
    )
}

#[component]
pub fn FullRegistration(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element{
    let mut loginState = use_context::<LoginState>();
    let mut loginError = use_signal(|| LoginError::None);
    rsx!(
        div { 
            class:"userRegistrationsContainer",
            div {
                class:"UserImageContainer",
                img {
                    class: "Profile_picture", // dosent need css
                    onclick: move |_| {
                        spawn(async move{
                            if let Some(blob) = choose_image().await {
                                loginState.pfp.set(blob);
                            }
                        });
                    }
                }
            }
            div { 
                class: "UserFieldsContainer",
                label { "Username:" },
                input{
                    id: "UserNameInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.username.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('EmailInput').focus();"#);
                    }
                }

                label { "Email:" },
                input{
                    id: "EmailInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.email.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('PasswordInput').focus();"#);
                    }
                }

                label { "Password:" },
                input{
                    id: "PasswordInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.password.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('PasswordAgainInput').focus();"#);
                    }
                }
                label { "Enter Password again:" },
                input{
                    id: "PasswordAgainInput",
                    r#type: "text",
                    oninput: move |evt| {
                        evt.prevent_default();
                        loginState.password_again.set(evt.value());
                    },

                    onkeydown: move |_| {
                        let _ = document::eval(r#"document.getElementById('Input').focus();"#);
                    }
                }

                div { 
                    id: "ErrorDiv",
                    match *loginError.read(){
                        LoginError::None => {
                            rsx!()
                        },
                        LoginError::PasswordNotSame => {
                            rsx!(
                                p { id:"LoginError",
                                    "Passwords do not match try again"
                                }
                            )
                        }
                        LoginError::EmailUnavailable => {
                            rsx!(
                                p { id:"LoginError",
                                    "Email already exists please login or try another email." //this should be unreachable on first use
                                }
                            )
                        }
                        LoginError::UsernameUnavailable => {
                            rsx!(
                                p { 
                                    id:"LoginError",
                                    "Username already exits please login or try another username."
                                }
                            )
                        }

                    }
                }
            }

            div { id: "ButtonsContainer",
                button { class: "submitButton",
                    onclick: move |_| {
                        spawn(async move {
                            let client = Client::new();
                            let username = loginState.username.read().to_string();
                            let email = loginState.email.read().to_string();
                            if let Ok(res) = client.get("http://localhost:3000/check_availability").json(
                                &CheckAvailability{
                                    username: username,
                                    email: email
                            }).send().await {
                                let results = res.json::<AvailabilityResponse>().await.unwrap_or(
                                    AvailabilityResponse { 
                                    username_available: false, email_available: false 
                                });
                                
                                if !results.username_available {
                                    loginError.set(LoginError::UsernameUnavailable);
                                    return;
                                }
                                else if !results.email_available {
                                    loginError.set(LoginError::EmailUnavailable);
                                    return;
                                }

                                else if loginState.password != loginState.password_again{
                                    loginError.set(LoginError::PasswordNotSame);
                                    return;
                                }

                                else {
                                    loginError.set(LoginError::None);
                                }

                                if *loginError.read() == LoginError::None
                                && let Ok(res2) = client.post("http://localhost:3000/Signup")
                                    .json(&SignUpStruct{
                                        user_email: loginState.email.to_string(),
                                        user_name: loginState.username.to_string(),
                                        user_password: loginState.password.to_string(),
                                        user_pfp: Some(loginState.pfp.to_string())
                                    }).send().await
                                && res2.status().is_success()
                                {
                                    on_next.call(());
                                }
                            }
                        });
                    },
                    "Submit"
                }

                button { 
                    class: "backButton",
                    onclick: move |_| {
                        on_back.call(());
                    },
                    "Go back"
                }
            }
        }
    )
}

#[component]
pub fn choose_update_scehdule() -> Element {
    rsx!(

    )
}