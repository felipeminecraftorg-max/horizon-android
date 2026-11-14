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
  const IDS=Object.freeze(Object.keys(PROFILES));
  const COLOR_FIELDS=Object.freeze(["bg","surface","accent","cool","warm"]);
  const merge=(base,id)=>{
    if(!Object.prototype.hasOwnProperty.call(PROFILES,id))return null;
    const p=PROFILES[id];
    return Object.assign({},base,p.colors,p.atmosphere,p.lighting,p.material,{paletteId:id});
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
      material:Object.freeze({...p.material})
    })];
  }));
  window.JadePalettes=Object.freeze({
    ids:IDS,profiles:Object.freeze(profiles),isPreset,
    apply:merge,contrast,colorFields:COLOR_FIELDS
  });
})();