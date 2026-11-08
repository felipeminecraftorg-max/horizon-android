// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.levelgen.structure.templatesystem;

import java.util.Optional;
import net.minecraft.resources.ResourceLocation;

/** The templates by id: the game reads them when one is placed. */
public class StructureTemplateManager {
    public static final StructureTemplateManager CURRENT = new StructureTemplateManager();

    public StructureTemplate getOrCreate(ResourceLocation id) {
        return new StructureTemplate(id);
    }

    public Optional<StructureTemplate> get(ResourceLocation id) {
        return Optional.of(new StructureTemplate(id));
    }
}
