// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.level;

import net.minecraft.world.level.Level;

public class ServerLevel extends Level implements net.minecraft.world.level.WorldGenLevel {
    @Override
    public ServerLevel getLevel() {
        return this;
    }

    public static final ServerLevel CURRENT = new ServerLevel();

    public net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager getStructureManager() {
        return net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager.CURRENT;
    }

    public net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }
}
