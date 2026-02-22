use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderPass {
    pub filename: String,
    pub is_main: bool,
    pub r#type: String,
    pub apply_on_children: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderPasses {
    pub passes: Vec<RenderPass>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum Effect {
    ApplyToAllPasses,
    AttachedObject {
        attachment_point: String,
        ao_file: String,
        ignore_errors: bool,
    },
    AttachedObjectBoneIndex {
        bone_index: u32,
        ao_file: String,
    },
    AttachedObjectEx {
        attachment_point: Option<String>,
        files: Vec<String>,
        uints1: [u32; 2],
        floats: [f32; 2],
        rotations: [Option<i32>; 2],
        include_aux: bool,
        ignore_errors: bool,
        multi_attach: bool,
    },
    ChildAttachedObject {
        ao_file: String,
        from_bone: String,
        from_bone_group_index: u32,
    },
    HideFirstPassAfterDelay {
        delay: f32,
    },
    HideFirstPassAfterDelayForDuration {
        delay: f32,
        duration: f32,
    },
    HideFirstPassUsingEPKParameter {
        parameter: String,
        float1: f32,
        float2: f32,
    },
    HideFirstPassUsingTimelineParameter {
        parameter: String,
        float1: f32,
        float2: f32,
    },
    ParentOnlyEffects,
    ParticleEffect {
        glob: String,
        pet_file: String,
        limit: Option<[u32; 2]>,
        ignore_errors: bool,
    },
    PlayMiscEffectPackAfterDelay {
        effect: String,
        delay: f32,
    },
    PlayMiscEffectPackOnBegin {
        effect: String,
    },
    PlayMiscEffectPackOnEnd {
        effect: String,
    },
    RenderPasses(RenderPasses),
    TrailEffect {
        glob: String,
        trl_file: String,
        limit: Option<[u32; 2]>,
        ignore_errors: bool,
    },

    /// Fallback for any unknown things
    Other {
        name: String,
        rest: String,
    },
}

pub type EPKFile = Vec<Effect>;
