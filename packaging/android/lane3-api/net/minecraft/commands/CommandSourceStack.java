// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.commands;

import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;

/** Where and as whom a command runs. */
public class CommandSourceStack {
    final Vec3 position;
    final Entity entity;
    final ServerLevel level;

    public CommandSourceStack(CommandSource source, Vec3 position, Vec2 rotation, ServerLevel level, int permission, String name, Component display, MinecraftServer server, Entity entity) {
        this.position = position == null ? Vec3.ZERO : position;
        this.entity = entity;
        this.level = level;
    }

    private CommandSourceStack(Vec3 position, Entity entity, ServerLevel level) {
        this.position = position;
        this.entity = entity;
        this.level = level;
    }

    public CommandSourceStack withSuppressedOutput() {
        return this;
    }

    public CommandSourceStack withPermission(int level) {
        return this;
    }

    public CommandSourceStack withMaximumPermission(int level) {
        return this;
    }

    public CommandSourceStack withPosition(Vec3 position) {
        return new CommandSourceStack(position, entity, level);
    }

    public CommandSourceStack withEntity(Entity entity) {
        return new CommandSourceStack(position, entity, level);
    }

    public CommandSourceStack withLevel(ServerLevel level) {
        return new CommandSourceStack(position, entity, level);
    }

    public Vec3 getPosition() {
        return position;
    }

    public Entity getEntity() {
        return entity;
    }

    public ServerLevel getLevel() {
        return level == null ? ServerLevel.CURRENT : level;
    }

    public MinecraftServer getServer() {
        return MinecraftServer.CURRENT;
    }
}
