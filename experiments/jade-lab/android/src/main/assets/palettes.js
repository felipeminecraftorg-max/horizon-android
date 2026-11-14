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
      colors:{bg:"#091611",surface:"#203c2c",accent:"#74da94",cool:"#63aeb8",warm:"#cb995a"},
      atmosphere:{fog:.48,waves:.33,vignette:.35,noise:.09,contrast:1.10,saturation:1.07},
      lighting:{azimuth:-38,elevation:.72,intensity:1.24,ambient:.66,edgeLight:.74,heroGlow:.79,bloom:.38},
      material:{roughness:.29,metallic:.31,reflectivity:.72,bevel:.57,depth:.65,ao:.43}
    },
    amethyst: {
      name:"Amethyst", tagline:"Ametista · roxo profundo e brilho de lavanda",
      colors:{bg:"#0d0a18",surface:"#30233e",accent:"#c39cff",cool:"#809de7",warm:"#eab9d9"},
      atmosphere:{fog:.59,waves:.22,vignette:.43,noise:.10,contrast:1.11,saturation:1.10},
      lighting:{azimuth:-53,elevation:.64,intensity:1.16,ambient:.63,edgeLight:.89,heroGlow:.92,bloom:.43},
      material:{roughness:.34,metallic:.28,reflectivity:.83,bevel:.62,depth:.77,ao:.51}
    },
    azure: {
      name:"Azure", tagline:"Azul glacial · superfícies cristalinas e névoa fria",
      colors:{bg:"#09141d",surface:"#1a3649",accent:"#86d3f4",cool:"#99e6e5",warm:"#e3b38c"},
      atmosphere:{fog:.44,waves:.25,vignette:.30,noise:.07,contrast:1.07,saturation:1.02},
      lighting:{azimuth:-24,elevation:.84,intensity:1.34,ambient:.74,edgeLight:.90,heroGlow:.66,bloom:.31},
      material:{roughness:.24,metallic:.35,reflectivity:.94,bevel:.54,depth:.60,ao:.35}
    },
    graphite: {
      name:"Graphite", tagline:"Cinza técnico · aço escovado e contraste frio",
      colors:{bg:"#0d1015",surface:"#252b35",accent:"#c6d0dc",cool:"#83a8c2",warm:"#d7b795"},
      atmosphere:{fog:.18,waves:.09,vignette:.47,noise:.15,contrast:1.19,saturation:.79},
      lighting:{azimuth:-45,elevation:.89,intensity:1.09,ambient:.63,edgeLight:.63,heroGlow:.47,bloom:.19},
      material:{roughness:.41,metallic:.62,reflectivity:.75,bevel:.44,depth:.70,ao:.57}
    },
    obsidian: {
      name:"Obsidian", tagline:"Preto profundo · luz branco-gelo e reflexo controlado",
      colors:{bg:"#050608",surface:"#1a2027",accent:"#edf1f8",cool:"#80abc9",warm:"#d6aa84"},
      atmosphere:{fog:.12,waves:.05,vignette:.63,noise:.09,contrast:1.22,saturation:.74},
      lighting:{azimuth:-57,elevation:.77,intensity:1.31,ambient:.51,edgeLight:1.07,heroGlow:.43,bloom:.22},
      material:{roughness:.27,metallic:.57,reflectivity:1.02,bevel:.59,depth:.53,ao:.62}
    },
    cobalt: {
      name:"Cobalt", tagline:"Azul elétrico refinado · energia e profundidade",
      colors:{bg:"#090f22",surface:"#213354",accent:"#91a7ff",cool:"#92c9fa",warm:"#dcb3dd"},
      atmosphere:{fog:.51,waves:.35,vignette:.38,noise:.08,contrast:1.16,saturation:1.09},
      lighting:{azimuth:-30,elevation:.68,intensity:1.33,ambient:.67,edgeLight:.98,heroGlow:1.04,bloom:.42},
      material:{roughness:.25,metallic:.43,reflectivity:1.01,bevel:.66,depth:.84,ao:.47}
    },
    smoke: {
      name:"Smoke", tagline:"Fumaça · grafite quente e cinza suave",
      colors:{bg:"#101013",surface:"#2c2b30",accent:"#d8cabd",cool:"#97a9b8",warm:"#d6aaa6"},
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
      backgroundStyle:"ribbons",backgroundIntensity:1.05,
      backgroundScale:1.12,backgroundFlow:.32,
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
      glassBlur:.69,glassTransmission:.78,glassTint:.35,
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
  const IDS=Object.freeze(Object.keys(PROFILES));
  const COLOR_FIELDS=Object.freeze(["bg","surface","accent","cool","warm"]);
  const merge=(base,id)=>{
    if(!Object.prototype.hasOwnProperty.call(PROFILES,id))return null;
    const p=PROFILES[id];
    return Object.assign({},base,p.colors,p.atmosphere,p.lighting,p.material,
      BACKDROPS[id],GLASS[id],{paletteId:id});
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
      glass:Object.freeze({...GLASS[id]})
    })];
  }));
  window.JadePalettes=Object.freeze({
    ids:IDS,profiles:Object.freeze(profiles),isPreset,
    apply:merge,contrast,colorFields:COLOR_FIELDS
  });
})();