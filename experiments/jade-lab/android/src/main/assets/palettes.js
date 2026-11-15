/* Curated DARK console identities for the Horizon Jade Lab.
 * A palette is not a color-swap: it carries calibrated light, atmosphere,
 * optical material and contrast. It never switches the app to light mode.
 * The chosen identity is serialized in every exported Jade Lab preset.
 */
(function () {
  "use strict";
  const PROFILES = {
    jade: {
      name:"Jade", tagline:"Console orgânico · luz viva e metal acetinado",
      colors:{bg:"#10181d",surface:"#263b41",accent:"#80d9c3",cool:"#91b7c5",warm:"#d98096"},
      atmosphere:{fog:.48,waves:.72,vignette:.35,noise:.09,contrast:1.10,saturation:1.07},
      lighting:{azimuth:-38,elevation:.72,intensity:1.24,ambient:.66,edgeLight:.74,heroGlow:.79,bloom:.38},
      material:{roughness:.29,metallic:.31,reflectivity:.72,bevel:.57,depth:.65,ao:.43}
    },
    amethyst: {
      name:"Amethyst", tagline:"Ametista · roxo profundo e brilho de lavanda",
      colors:{bg:"#201d2c",surface:"#423d51",accent:"#c5a4ed",cool:"#a9b8da",warm:"#cceda4"},
      atmosphere:{fog:.59,waves:.22,vignette:.43,noise:.10,contrast:1.11,saturation:1.10},
      lighting:{azimuth:-53,elevation:.64,intensity:1.16,ambient:.63,edgeLight:.89,heroGlow:.92,bloom:.43},
      material:{roughness:.34,metallic:.28,reflectivity:.83,bevel:.62,depth:.77,ao:.51}
    },
    azure: {
      name:"Azure", tagline:"Azul glacial · superfícies cristalinas e névoa fria",
      colors:{bg:"#1a2832",surface:"#34495a",accent:"#8fd3eb",cool:"#b9dfe7",warm:"#eba78f"},
      atmosphere:{fog:.44,waves:.25,vignette:.30,noise:.07,contrast:1.07,saturation:1.02},
      lighting:{azimuth:-24,elevation:.84,intensity:1.34,ambient:.74,edgeLight:.90,heroGlow:.66,bloom:.31},
      material:{roughness:.24,metallic:.35,reflectivity:.94,bevel:.54,depth:.60,ao:.35}
    },
    graphite: {
      name:"Graphite", tagline:"Cinza técnico · aço escovado e contraste frio",
      colors:{bg:"#151a1e",surface:"#394249",accent:"#cad7dc",cool:"#9bacb6",warm:"#d9c8b3"},
      atmosphere:{fog:.18,waves:.09,vignette:.47,noise:.15,contrast:1.19,saturation:.79},
      lighting:{azimuth:-45,elevation:.89,intensity:1.09,ambient:.63,edgeLight:.63,heroGlow:.47,bloom:.19},
      material:{roughness:.41,metallic:.62,reflectivity:.75,bevel:.44,depth:.70,ao:.57}
    },
    obsidian: {
      name:"Obsidian", tagline:"Preto profundo · luz branco-gelo e reflexo controlado",
      colors:{bg:"#080c12",surface:"#272e36",accent:"#f0f0e8",cool:"#95acbd",warm:"#d5bfa5"},
      atmosphere:{fog:.12,waves:.05,vignette:.63,noise:.09,contrast:1.22,saturation:.74},
      lighting:{azimuth:-57,elevation:.77,intensity:1.31,ambient:.51,edgeLight:1.07,heroGlow:.43,bloom:.22},
      material:{roughness:.27,metallic:.57,reflectivity:1.02,bevel:.59,depth:.53,ao:.62}
    },
    cobalt: {
      name:"Cobalt", tagline:"Azul elétrico refinado · energia e profundidade",
      colors:{bg:"#19253d",surface:"#3a4964",accent:"#a5b9ed",cool:"#a6cce6",warm:"#edd9a5"},
      atmosphere:{fog:.51,waves:.35,vignette:.38,noise:.08,contrast:1.16,saturation:1.09},
      lighting:{azimuth:-30,elevation:.68,intensity:1.33,ambient:.67,edgeLight:.98,heroGlow:1.04,bloom:.42},
      material:{roughness:.25,metallic:.43,reflectivity:1.01,bevel:.66,depth:.84,ao:.47}
    },
    smoke: {
      name:"Smoke", tagline:"Fumaça · grafite quente e cinza suave",
      colors:{bg:"#242329",surface:"#454249",accent:"#dfd0c4",cool:"#abb4bd",warm:"#c4d3df"},
      atmosphere:{fog:.26,waves:.07,vignette:.41,noise:.18,contrast:1.09,saturation:.79},
      lighting:{azimuth:-61,elevation:.74,intensity:1.02,ambient:.73,edgeLight:.55,heroGlow:.37,bloom:.16},
      material:{roughness:.52,metallic:.23,reflectivity:.57,bevel:.42,depth:.63,ao:.40}
    }
  };
  // Palette-specific landscape art direction, not merely an RGB tint.
  // Presets own the compositional style, halo placement and band hierarchy;
  // the user can still override every number independently in Fundo.
  const BACKDROPS={
    jade:{
      backgroundStyle:"ribbons",backgroundIntensity:1.18,
      backgroundScale:1.12,backgroundFlow:.75,
      ribbonLayers:3,ribbonCurve:.95,ribbonWidth:.73,ribbonSoftness:1.10,
      ribbonGleam:.72,backgroundGlow:.78,gradientDepth:1.05,
      backgroundContrast:.75,backgroundGrain:.16,
      backgroundCenterX:.34,backgroundCenterY:.58,backgroundParallax:.37
    },
    amethyst:{
      backgroundStyle:"aurora",backgroundIntensity:1.22,
      backgroundScale:.86,backgroundFlow:.20,
      ribbonLayers:4,ribbonCurve:1.27,ribbonWidth:1.16,ribbonSoftness:1.48,
      ribbonGleam:.46,backgroundGlow:1.13,gradientDepth:1.19,
      backgroundContrast:1.14,backgroundGrain:.13,
      backgroundCenterX:.45,backgroundCenterY:.44,backgroundParallax:.23
    },
    azure:{
      backgroundStyle:"satin",backgroundIntensity:1.08,
      backgroundScale:1.37,backgroundFlow:.25,
      ribbonLayers:3,ribbonCurve:.68,ribbonWidth:.73,ribbonSoftness:.88,
      ribbonGleam:1.10,backgroundGlow:.85,gradientDepth:.89,
      backgroundContrast:1.03,backgroundGrain:.075,
      backgroundCenterX:.63,backgroundCenterY:.36,backgroundParallax:.51
    },
    graphite:{
      backgroundStyle:"minimal",backgroundIntensity:.52,
      backgroundScale:1.18,backgroundFlow:.07,
      ribbonLayers:2,ribbonCurve:.35,ribbonWidth:.48,ribbonSoftness:1.18,
      ribbonGleam:.32,backgroundGlow:.44,gradientDepth:.68,
      backgroundContrast:.40,backgroundGrain:.27,
      backgroundCenterX:.62,backgroundCenterY:.28,backgroundParallax:.12
    },
    obsidian:{
      backgroundStyle:"halo",backgroundIntensity:.86,
      backgroundScale:.97,backgroundFlow:.10,
      ribbonLayers:3,ribbonCurve:.38,ribbonWidth:.59,ribbonSoftness:1.36,
      ribbonGleam:.76,backgroundGlow:.68,gradientDepth:1.26,
      backgroundContrast:.59,backgroundGrain:.10,
      backgroundCenterX:.61,backgroundCenterY:.43,backgroundParallax:.19
    },
    cobalt:{
      backgroundStyle:"ribbons",backgroundIntensity:1.24,
      backgroundScale:1.32,backgroundFlow:.44,
      ribbonLayers:4,ribbonCurve:1.29,ribbonWidth:.83,ribbonSoftness:.93,
      ribbonGleam:1.15,backgroundGlow:1.10,gradientDepth:1.16,
      backgroundContrast:1.02,backgroundGrain:.12,
      backgroundCenterX:.35,backgroundCenterY:.61,backgroundParallax:.56
    },
    smoke:{
      backgroundStyle:"satin",backgroundIntensity:.66,
      backgroundScale:.83,backgroundFlow:.08,
      ribbonLayers:2,ribbonCurve:.39,ribbonWidth:.92,ribbonSoftness:1.75,
      ribbonGleam:.44,backgroundGlow:.40,gradientDepth:.72,
      backgroundContrast:.47,backgroundGrain:.32,
      backgroundCenterX:.46,backgroundCenterY:.31,backgroundParallax:.10
    }
  };
  // Optical personality complements each palette's atmosphere. These are
  // independent visual decisions, not simple color replacements.
  const GLASS={
    jade:{glassStyle:"frosted",glassQuality:"balanced",glassOpacity:.38,
      glassBlur:.69,glassTransmission:.48,glassTint:.35,
      glassScatter:.42,glassFresnel:.58,glassRim:.64,
      glassGrain:.15,glassHighlight:.62,glassShadow:.52},
    amethyst:{glassStyle:"tinted",glassQuality:"balanced",glassOpacity:.31,
      glassBlur:.84,glassTransmission:.89,glassTint:.73,
      glassScatter:.72,glassFresnel:.76,glassRim:.88,
      glassGrain:.11,glassHighlight:.76,glassShadow:.48},
    azure:{glassStyle:"frosted",glassQuality:"balanced",glassOpacity:.29,
      glassBlur:.57,glassTransmission:.91,glassTint:.29,
      glassScatter:.32,glassFresnel:.94,glassRim:.89,
      glassGrain:.07,glassHighlight:.97,glassShadow:.40},
    graphite:{glassStyle:"matte",glassQuality:"fast",glassOpacity:.68,
      glassBlur:.37,glassTransmission:.31,glassTint:.40,
      glassScatter:.21,glassFresnel:.45,glassRim:.39,
      glassGrain:.29,glassHighlight:.40,glassShadow:.72},
    obsidian:{glassStyle:"tinted",glassQuality:"balanced",glassOpacity:.51,
      glassBlur:.62,glassTransmission:.65,glassTint:.37,
      glassScatter:.17,glassFresnel:1.13,glassRim:1.01,
      glassGrain:.11,glassHighlight:.81,glassShadow:.77},
    cobalt:{glassStyle:"tinted",glassQuality:"balanced",glassOpacity:.32,
      glassBlur:.80,glassTransmission:.86,glassTint:.62,
      glassScatter:.51,glassFresnel:.85,glassRim:.94,
      glassGrain:.12,glassHighlight:.78,glassShadow:.51},
    smoke:{glassStyle:"matte",glassQuality:"fast",glassOpacity:.69,
      glassBlur:.43,glassTransmission:.29,glassTint:.54,
      glassScatter:.16,glassFresnel:.37,glassRim:.31,
      glassGrain:.33,glassHighlight:.33,glassShadow:.69}
  };
  // Scene colors are a SECOND system independent from each accent.
  // Pearl, silver and brushed metal are chosen to harmonize with every theme.
  const SCENE_TONES={
    jade:{sceneNeutral:"#899b9e",sceneMetal:"#586a72",sceneHighlight:"#e5e3d6",
      environmentLight:.40,environmentWarmth:.40},
    amethyst:{sceneNeutral:"#888b9d",sceneMetal:"#615b73",sceneHighlight:"#e8deea",
      environmentLight:.80,environmentWarmth:.25},
    azure:{sceneNeutral:"#879da9",sceneMetal:"#597584",sceneHighlight:"#e5edf0",
      environmentLight:.92,environmentWarmth:.28},
    graphite:{sceneNeutral:"#8c9396",sceneMetal:"#5f6b72",sceneHighlight:"#e7e8de",
      environmentLight:.77,environmentWarmth:.22},
    obsidian:{sceneNeutral:"#78838a",sceneMetal:"#454e59",sceneHighlight:"#f1f1e9",
      environmentLight:.60,environmentWarmth:.19},
    cobalt:{sceneNeutral:"#889bad",sceneMetal:"#536987",sceneHighlight:"#e4e6eb",
      environmentLight:.86,environmentWarmth:.27},
    smoke:{sceneNeutral:"#93898b",sceneMetal:"#696166",sceneHighlight:"#eee2d5",
      environmentLight:.78,environmentWarmth:.41}
  };
  const SCENE_COLOR_FIELDS=Object.freeze(["sceneNeutral","sceneMetal","sceneHighlight"]);
  const IDS=Object.freeze(Object.keys(PROFILES));
  const COLOR_FIELDS=Object.freeze(["bg","surface","accent","cool","warm"]);
  const merge=(base,id)=>{
    if(!Object.prototype.hasOwnProperty.call(PROFILES,id))return null;
    const p=PROFILES[id];
    return Object.assign({},base,p.colors,p.atmosphere,p.lighting,p.material,
      BACKDROPS[id],GLASS[id],SCENE_TONES[id],{paletteId:id});
  };
  const luminance=hex=>{
    const rgb=[1,3,5].map(i=>parseInt(hex.substring(i,i+2),16)/255);
    const t=rgb.map(v=>v<=.04045?v/12.92:Math.pow((v+.055)/1.055,2.4));
    return .2126*t[0]+.7152*t[1]+.0722*t[2];
  };
  const contrast=(a,b)=>{
    const x=luminance(a),y=luminance(b);
    return (Math.max(x,y)+.05)/(Math.min(x,y)+.05);
  };
  const isPreset=id=>Object.prototype.hasOwnProperty.call(PROFILES,id);
  const profiles=Object.fromEntries(IDS.map(id=>{
    const p=PROFILES[id];
    return [id,Object.freeze({
      ...p,
      colors:Object.freeze({...p.colors}),
      atmosphere:Object.freeze({...p.atmosphere}),
      lighting:Object.freeze({...p.lighting}),
      material:Object.freeze({...p.material}),
      backdrop:Object.freeze({...BACKDROPS[id]}),
      glass:Object.freeze({...GLASS[id]}),
      scene:Object.freeze({...SCENE_TONES[id]})
    })];
  }));
  window.JadePalettes=Object.freeze({
    ids:IDS,profiles:Object.freeze(profiles),isPreset,
    apply:merge,contrast,colorFields:COLOR_FIELDS,sceneColorFields:SCENE_COLOR_FIELDS
  });
})();