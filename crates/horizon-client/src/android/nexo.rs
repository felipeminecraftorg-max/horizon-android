//! Local-only transport for the native Old School view. Work stays on the
//! existing Rust launcher thread; JavaScript can enqueue only typed UI actions.
use horizon_client::launcher_nexo::{Command, parse};
use jni::{JNIEnv, NativeMethod, JavaVM, objects::{JObject,JString,JValue}};
use std::{collections::VecDeque,sync::{Mutex,atomic::{AtomicBool,Ordering}}};

static COMMANDS: Mutex<VecDeque<Command>> = Mutex::new(VecDeque::new());
static ACTIVE: AtomicBool = AtomicBool::new(false);
pub fn active() -> bool { ACTIVE.load(Ordering::Acquire) }
pub fn take() -> Vec<Command> { COMMANDS.lock().unwrap_or_else(|e|e.into_inner()).drain(..).collect() }

#[unsafe(export_name="Java_dev_horizon_android_HorizonActivity_nativeNexoRequest")]
pub extern "system" fn request(mut env: JNIEnv<'_>, _: JObject<'_>, value: JString<'_>) -> jni::sys::jboolean {
    if !active() || value.is_null() { return 0; }
    let Ok(input) = env.get_string(&value).map(String::from) else { let _=env.exception_clear(); return 0; };
    let Ok(command) = parse(&input) else { return 0; };
    let mut queue=COMMANDS.lock().unwrap_or_else(|e|e.into_inner());
    if queue.len()>=64 { return 0; } queue.push_back(command); 1
}
fn activity(f: impl FnOnce(&mut JNIEnv<'_>,&JObject<'_>)->jni::errors::Result<()>) -> bool {
    crate::APP.with(|slot| {
        let app=slot.borrow();let Some(app)=app.as_ref() else{return false;};
        let Ok(vm)=(unsafe{JavaVM::from_raw(app.vm_as_ptr().cast())}) else{return false;};
        let Ok(mut env)=vm.attach_current_thread() else{return false;};
        let result=env.with_local_frame(16,|env| { let object=unsafe{JObject::from_raw(app.activity_as_ptr().cast())}; f(env,&object) });
        if let Err(error)=result { let _=env.exception_clear(); log::warn!("Old School view transport: {error}");false } else {true}
    })
}
pub fn initialize() -> bool {
    activity(|env,object| {let class=env.get_object_class(object)?;env.register_native_methods(class,&[NativeMethod{name:"nativeNexoRequest".into(),sig:"(Ljava/lang/String;)Z".into(),fn_ptr:request as *mut std::ffi::c_void}])})
}
pub fn show(value: bool) -> bool {
    ACTIVE.store(value,Ordering::Release);
    if !value {COMMANDS.lock().unwrap_or_else(|e|e.into_inner()).clear();}
    activity(|env,object| {env.call_method(object,"horizonNexo","(Z)V",&[JValue::Bool(u8::from(value))])?;Ok(())})
}
pub fn publish(value: &str) {
    if !active(){return;}
    activity(|env,object| {let text=JObject::from(env.new_string(value)?);env.call_method(object,"horizonNexoState","(Ljava/lang/String;)V",&[JValue::Object(&text)])?;Ok(())});
}
