//! Cross-module smoke tests for the native Fresh Animations resource-data pipeline.
//! These verify parsing and deterministic expressions, not graphics or APK behavior.
use horizon_optifine::{entity_animations, entity_models, entity_textures};

#[test]
fn reads_cem_jpm_animation_and_entity_variation() {
    let model = entity_models::parse_with(
        r#"{"texture":"textures/entity/zombie/zombie.png","textureSize":[64,64],
            "models":[{"part":"head","id":"head","model":"zombie_head.jpm",
                "animations":[{"head.rx":"sin(limb_swing)*0.5 + if(is_baby,0.2,0)"}]}]}"#,
        |file| (file=="zombie_head.jpm").then(|| r#"{"boxes":[
            {"coordinates":[-4,-8,-4,8,8,8],"textureOffset":[0,0]}]}"#.to_string()),
    ).expect("CEM and JPM must parse");
    let head = &model.parts[0];
    assert_eq!(head.boxes.len(), 1);
    let (target, expression) = &head.animation[0];
    let evaluator = entity_animations::Animation::compile(target,expression).unwrap();
    let rotation = evaluator.sample(&|name| match name {
        "limb_swing" => Some(0.0), "is_baby" => Some(1.0), _ => None,
    }).unwrap();
    assert!((rotation - 0.2).abs() < 0.00001);

    let rules = entity_textures::parse(
        "textures.1=1 2\nweights.1=1 4\nname.1=pattern:Zombie*"
    ).unwrap();
    let context = entity_textures::EntityContext {
        seed: 42, name: Some("Zombie"), biome: None, block_y: None,
    };
    let first = rules.select(context).unwrap();
    assert!(first == 1 || first == 2);
    assert_eq!(rules.select(context), Some(first));
}

#[test]
fn unsafe_or_unsupported_resources_fail_closed() {
    assert!(entity_models::parse_with(
        r#"{"models":[{"model":"../../outside.jpm"}]}"#, |_| None
    ).is_err());
    let rules = entity_textures::parse(
        "textures.1=2\nprofessions.1=farmer"
    ).unwrap();
    assert!(rules.rules.is_empty());
    assert!(!rules.skipped.is_empty());
    assert!(entity_animations::Compiled::parse("sin(1);system()").is_err());
}

#[test]
fn numbers_and_boolean_conditions_use_the_supplied_render_state() {
    let e = entity_animations::Compiled::parse(
        "if(age > 10 && is_in_water, sin(head_yaw), cos(head_yaw))"
    ).unwrap();
    let value = e.eval(&|name| match name {
        "age" => Some(11.0), "is_in_water" => Some(1.0),
        "head_yaw" => Some(0.0), _ => None,
    }).unwrap();
    assert_eq!(value, 0.0);
}
