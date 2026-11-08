// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server;

import dev.horizon.android.lane3.Native;

public class MinecraftServer {
    public static final MinecraftServer CURRENT = new MinecraftServer();

    public int getTickCount() {
        return (int) Native.time();
    }

    public net.minecraft.commands.Commands getCommands() {
        return net.minecraft.commands.Commands.CURRENT;
    }

    public net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager getStructureManager() {
        return net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager.CURRENT;
    }

    public boolean isDedicatedServer() {
        return false;
    }

    public net.minecraft.server.level.ServerLevel overworld() {
        return net.minecraft.server.level.ServerLevel.CURRENT;
    }
}
