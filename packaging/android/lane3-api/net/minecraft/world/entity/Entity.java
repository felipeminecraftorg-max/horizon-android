// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity;

import dev.horizon.android.lane3.Native;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;

/** The player an event acts for, through Horizon. */
public abstract class Entity {
    /** What mods keep on this entity (NeoForge's attachments), by type. */
    private final java.util.Map<Object, Object> attachments = new java.util.HashMap<>();

    public Level level() {
        return ServerLevel.CURRENT;
    }

    private static double at(int i) {
        String[] p = dev.horizon.android.lane3.Neo.player();
        try {
            return p.length > i ? Double.parseDouble(p[i]) : 0;
        } catch (NumberFormatException e) {
            return 0;
        }
    }

    public double getX() {
        return at(0);
    }

    public double getY() {
        return at(1);
    }

    public double getZ() {
        return at(2);
    }

    public float getYRot() {
        return (float) at(3);
    }

    public float getXRot() {
        return (float) at(4);
    }

    public net.minecraft.world.phys.Vec3 position() {
        return new net.minecraft.world.phys.Vec3(getX(), getY(), getZ());
    }

    public net.minecraft.core.BlockPos blockPosition() {
        return new net.minecraft.core.BlockPos((int) Math.floor(getX()), (int) Math.floor(getY()), (int) Math.floor(getZ()));
    }

    public net.minecraft.world.phys.Vec2 getRotationVector() {
        return new net.minecraft.world.phys.Vec2(getXRot(), getYRot());
    }

    public net.minecraft.network.chat.Component getName() {
        return net.minecraft.network.chat.Component.literal("Player");
    }

    public net.minecraft.network.chat.Component getDisplayName() {
        return getName();
    }

    public net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }

    public int getId() {
        return 0;
    }

    /** Moves the entity in its level (a command: Horizon moves the player). */
    public void teleportTo(double x, double y, double z) {
        Native.command("tp @s " + x + " " + y + " " + z);
    }

    /** NeoForge's attachments: the data of that type this entity holds, made now if none. */
    @SuppressWarnings("unchecked")
    public <T> T getData(java.util.function.Supplier<net.neoforged.neoforge.attachment.AttachmentType<T>> type) {
        net.neoforged.neoforge.attachment.AttachmentType<T> t = type.get();
        return (T) attachments.computeIfAbsent(t, k -> t.horizonMake(this));
    }

    @SuppressWarnings("unchecked")
    public <T> T setData(java.util.function.Supplier<net.neoforged.neoforge.attachment.AttachmentType<T>> type, T value) {
        return (T) attachments.put(type.get(), value);
    }

    public boolean hasData(java.util.function.Supplier<? extends net.neoforged.neoforge.attachment.AttachmentType<?>> type) {
        return attachments.containsKey(type.get());
    }

    public boolean isShiftKeyDown() {
        return Native.sneaking();
    }

    public boolean isCrouching() {
        return Native.sneaking();
    }

    public boolean isSpectator() {
        return false;
    }
}
