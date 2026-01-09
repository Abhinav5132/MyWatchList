use anyhow::anyhow;

pub use crate::frontend::*;
use crate::{
    backend::sign_up::check_availability,
    frontend::{
        self,
        login_popup::{Login, LoginStruct, SignUpStruct, store_refresh_token},
        manage_user_profile::{ChangePfp, choose_image},
    },
};

pub enum Steps {
    WELCOME,
    APPTYPE,
    USEREGISTRATION, // configure sync server should always be before user registration. As we check if the user is registered on that specific sync server.
    CONFIGURESYNCSERVER, // only encountered if they choose PERSONALSYNC OR PUBLICSYNC
    CHOOSEUPDATESCHEDULE,
    FINALIZE,
}

pub enum AccountType {
    LOGIN,
    REGISTER,
    LOCAL,
    TOBEDETERMINED,
}

#[derive(Debug)]
pub enum AppType {
    LOCAL,
    PERSONALSYNC, // personal sync should guide you though installing and setting up your own sync server.
    PUBLICSYNC,   // act the same way for now
}
#[derive(Clone)]
struct OnboardingState {
    app_type: Signal<AppType>,
    acc_type: Signal<AccountType>,
    update_schedule: Signal<UpdateScehdule>,
    login_state: LoginState,
}

#[derive(Clone)]
pub enum UpdateScehdule {
    OnStartUp,
    OnceADay,
    OnceAWeek,
    OnceAMonth,
    Never,
}

impl UpdateScehdule {
    pub fn as_string(&mut self) -> String {
        match self {
            Self::OnStartUp => "ONSTARTUP".to_string(),
            Self::OnceADay => "ONCEADAY".to_string(),
            Self::OnceAWeek => "ONCEAWEEK".to_string(),
            Self::OnceAMonth => "ONCEAMONTH".to_string(),
            Self::Never => "NEVER".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum LoginError {
    PasswordNotSame,
    EmailUnavailable,
    UsernameUnavailable,
    None,
}

#[derive(Clone)]
pub struct LoginState {
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

// TODO login should not be asked for update schedule
//TODO login state should persist on going back and should not allow blank proceeds
#[component]
pub fn FirstTimePage() -> Element {
    let mut step = use_signal(|| Steps::WELCOME);
    let mut app_type = use_signal(|| AppType::LOCAL);
    let mut acc_type = use_signal(|| AccountType::LOCAL);
    let mut update_schedule = use_signal(|| UpdateScehdule::Never);
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut password_again = use_signal(|| "".to_string());
    let mut pfp = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut login_state = LoginState {
        username,
        password,
        email,
        password_again,
        pfp,
    };

    let navigator = use_navigator();
    provide_context(OnboardingState {
        app_type,
        acc_type,
        update_schedule,
        login_state,
    });

    if !(*USERNAME.read()).is_empty() {
        navigator.push(crate::frontend::router::routes::HomePage {});
    }
    rsx!(match *step.read() {
        Steps::WELCOME => rsx!(WelcomePage {
            on_next: move || step.set(Steps::APPTYPE),
        }),
        Steps::APPTYPE => rsx!(SelectAppType {
            on_next: move || step.set(Steps::CONFIGURESYNCSERVER),
            on_back: move || step.set(Steps::WELCOME)
        }),

        Steps::USEREGISTRATION => {
            rsx!(UserRegistrations {
                on_next: move || step.set(Steps::CHOOSEUPDATESCHEDULE),
                on_back: move || step.set(Steps::CONFIGURESYNCSERVER)
            })
        }
        Steps::CONFIGURESYNCSERVER => {
            match *app_type.read() {
                AppType::LOCAL => {
                    rsx!(ConfigureSyncServerLocal {
                        on_next: move || step.set(Steps::USEREGISTRATION),
                        on_back: move || step.set(Steps::APPTYPE)
                    })
                }
                AppType::PERSONALSYNC => {
                    rsx!(ConfigureSyncServerPersonal {
                        on_next: move || step.set(Steps::USEREGISTRATION),
                        on_back: move || step.set(Steps::APPTYPE)
                    })
                }

                AppType::PUBLICSYNC => {
                    rsx!(ConfigureSyncServerPrivate {
                        on_next: move || step.set(Steps::USEREGISTRATION),
                        on_back: move || step.set(Steps::APPTYPE)
                    })
                }
            }
        }
        Steps::CHOOSEUPDATESCHEDULE => {
            rsx!(choose_update_scehdule {
                on_next: move || {
                    step.set(Steps::FINALIZE);
                },
                on_back: move || {
                    step.set(Steps::USEREGISTRATION);
                }
            })
        }

        Steps::FINALIZE => {
            rsx!(FinalPage {
                on_back: move || step.set(Steps::CHOOSEUPDATESCHEDULE),
                on_next: move || {
                    navigator.push(crate::frontend::router::routes::HomePage {});
                }
            })
        }
    })
}

#[component]
pub fn WelcomePage(on_next: EventHandler<()>) -> Element {
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
pub fn SelectAppType(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
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
pub fn ConfigureSyncServerPrivate(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    rsx!()
}

#[component]
pub fn ConfigureSyncServerPersonal(
    on_next: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    rsx!()
}

#[component]
pub fn ConfigureSyncServerLocal(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
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
pub fn UserRegistrations(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    let mut state = use_context::<OnboardingState>();

    rsx!(match *state.acc_type.read() {
        AccountType::LOCAL => {
            rsx!(FullRegistration {
                on_next: move || on_next.call(()),
                on_back: move || on_back.call(())
            })
        }

        AccountType::LOGIN => {
            rsx!(LoginRegistrations {
                on_next: move || on_next.call(()),
                on_back: move || on_back.call(())
            })
        }

        AccountType::REGISTER => {
            rsx!(FullRegistration {
                on_next: move || on_next.call(()),
                on_back: move || on_back.call(())
            })
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
    })
}

#[component]
pub fn LoginRegistrations(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    let mut loginState = use_context::<OnboardingState>().login_state;
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('EmailInput').focus();"#);
                        }
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('PasswordAgainInput').focus();"#);
                        }
                    }
                }
            }

            div { id: "ButtonsContainer",
                button { class: "submitButton",
                    onclick: move |_| {
                        on_next.call(());
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

pub async fn login_spawn(username: String, password: String) -> anyhow::Result<()> {
    let client = Client::new();

    match client
        .post("http://localhost:3000/login")
        .json(&LoginStruct {
            username: username.clone(),
            password,
        })
        .send()
        .await
    {
        Ok(res) => {
            match res.status().is_success() {
                true => {
                    if let Ok(auth_response) = res.json::<AuthResponse>().await {
                        *TOKEN.write() = auth_response.access_token;
                        *REFRESHIN.write() = auth_response.expires_in as i64;
                        *USERNAME.write() = username.clone();
                        let _ =
                            store_refresh_token(&username, auth_response.refresh_token.as_str());
                        // do something with this status later.
                        let path = storage_file();
                        match fs::write(path, username) {
                            Ok(a) => {
                                print!("Successfull wrote the token to");
                                a
                            }
                            Err(e) => {
                                dbg!("Failed to write token to the disk");
                                dbg!(e);
                            }
                        }
                        get_userid_from_jwt();
                    }
                    Ok(())
                }
                false => Err(anyhow!("IDK BRO")),
            }
        }
        Err(e) => {
            dbg!(&e);
            Err(anyhow!(e.to_string()))
        }
    }
}

pub async fn sign_up_spawn(
    login_state: LoginState,
    mut update_schedule: UpdateScehdule,
) -> anyhow::Result<()> {
    let client = Client::new();
    let name = login_state.username.read().to_string();
    let email = login_state.email.read().to_string();
    let pwd = login_state.password.read().to_string();
    let pfp = Some(login_state.pfp.read().to_string());
    match client
        .post("http://localhost:3000/Signup")
        .json(&SignUpStruct {
            user_email: email,
            user_name: name.clone(),
            user_password: pwd,
            user_pfp: pfp,
            chosen_update_schedule: update_schedule.as_string(),
        })
        .send()
        .await
    {
        Ok(res) => {
            match res.status().is_success() {
                true => {
                    if let Ok(auth_response) = res.json::<AuthResponse>().await {
                        *TOKEN.write() = auth_response.access_token;
                        *REFRESHIN.write() = auth_response.expires_in as i64;
                        *USERNAME.write() = name.clone();
                        let _ = store_refresh_token(&name, auth_response.refresh_token.as_str());
                        // do something with this status later.
                        let path = storage_file();
                        match fs::write(path, name) {
                            Ok(a) => {
                                print!("Successfull wrote the token to");
                                a
                            }
                            Err(e) => {
                                dbg!("Failed to write token to the disk");
                                dbg!(e);
                            }
                        }
                        get_userid_from_jwt();
                    }
                    Ok(())
                }
                false => Err(anyhow!("IDK BRO")),
            }
        }
        Err(e) => {
            dbg!(&e);
            Err(anyhow!(e.to_string()))
        }
    }
}

#[component]
pub fn FullRegistration(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    let mut loginState = use_context::<OnboardingState>().login_state;
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('EmailInput').focus();"#);
                        }
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('PasswordInput').focus();"#);
                        }
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('PasswordAgainInput').focus();"#);
                        }
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

                    onkeydown: move |event| {
                        if event.code().to_string() == "Enter"{
                            let _ = document::eval(r#"document.getElementById('Input').focus();"#);
                        }
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
                                    username: username.clone(),
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

                                else if *loginState.password.read() != *loginState.password_again.read(){
                                    loginError.set(LoginError::PasswordNotSame);
                                    return;
                                }

                                else {
                                    loginError.set(LoginError::None);
                                }

                                if *loginError.read() == LoginError::None
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
pub fn choose_update_scehdule(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    let mut state = use_context::<OnboardingState>();
    rsx!(
        div {
            class: "SelectUpdateScheduleContainer",
            h3 {
                class: "FirstTimeQuestion",
                "Select how often you want the app data to update"
            }

            div {
                class: "Button_wrapapper",
                div {
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.update_schedule.set(UpdateScehdule::OnStartUp);
                        on_next.call(());
                    },
                    "Update on startup"
                }
                span {
                    class: "selectAppTypeButtonToolTip",
                    "The app will update when opened."
                }
            }
            div {
                class: "Button_wrapapper",
                div {
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.update_schedule.set(UpdateScehdule::OnceADay);
                        on_next.call(());
                    },
                    "Once a day"
                }

                span {
                    class: "selectAppTypeButtonToolTip",
                    "The app will update every 24 hrs"
                }
            }
            div {
                class: "Button_wrapapper",
                div {
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.update_schedule.set(UpdateScehdule::OnceAWeek);
                        on_next.call(());
                    },
                    "Once a week."
                }

                span {
                    class: "selectAppTypeButtonToolTip",
                    "App will update 7 days after the previous update."
                }
            }

            div {
                class: "Button_wrapapper",
                div {
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.update_schedule.set(UpdateScehdule::OnceAMonth);
                        on_next.call(());
                    },
                    "Once a Month"
                }

                span {
                    class: "selectAppTypeButtonToolTip",
                    "App will update 30 days after the previous update."
                }
            }

            div {
                class: "Button_wrapapper",
                div {
                    class: "selectAppTypeButton",
                    onclick: move |_|{
                        state.update_schedule.set(UpdateScehdule::Never);
                        on_next.call(());
                    },
                    "Never"
                }

                span {
                    class: "selectAppTypeButtonToolTip",
                    "App can only be updated manually."
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
pub fn FinalPage(on_next: EventHandler<()>, on_back: EventHandler<()>) -> Element {
    let state = use_context::<OnboardingState>();
    let login = state.login_state.clone();
    let update_schedule = state.update_schedule.read().to_owned().as_string();

    rsx!(
        div {
            class: "FinalizeContainer",

            h3 {
                class: "FinalizeHeading",
                "Please review your entered details"
            }

            div {
                class: "FinalizedDetailsContainer",

                div { class: "DetailCard",
                    h4 { "Username" }
                    p { "{login.username.read()}" }
                }

                div { class: "DetailCard",
                    h4 { "Email" }
                    p { "{login.email.read()}" }
                }

                if !login.pfp.read().is_empty() {
                    div { class: "DetailCard",
                        h4 { "Profile Picture" }
                        img {
                            src: "{login.pfp.read()}",
                            class: "ProfilePreview"
                        }
                    }
                }

                div { class: "DetailCard",
                    h4 { "Application Type" }
                    p {
                        "{state.app_type.read():?}"
                    }
                }

                div { class: "DetailCard",
                    h4 { "Update Frequency" }
                    p {
                        "{update_schedule}"
                    }
                }
            }

            div {
                class: "FinalizeButtons",

                button {
                    class: "BackButton",
                    onclick: move |_| on_back.call(()),
                    "Back"
                }

                button {
                    class: "ConfirmButton",
                    onclick: move |_| {
                        match *state.acc_type.read() {
                            AccountType::LOGIN => {
                                spawn(async move {
                                    let username = login.username.read().to_string();
                                    let password = login.password.read().to_string();
                                    let _ = login_spawn(username, password).await;
                                    on_next.call(());
                                });

                            },
                            AccountType::LOCAL | AccountType::REGISTER => {
                                let login_a = login.clone();
                                let update_schedule = state.update_schedule.read().cloned();
                                spawn(async move {
                                match sign_up_spawn(login_a, update_schedule).await {
                                    Ok(a) => {
                                        on_next.call(());
                                    },
                                    Err(e) => {
                                        dbg!(e);
                                    }
                                 }
                                });
                            }
                            AccountType::TOBEDETERMINED => {
                                // this should not be possible
                                panic!()
                            }
                        }




                    },
                    "Confirm & Finish"
                }
            }
        }
    )
}
