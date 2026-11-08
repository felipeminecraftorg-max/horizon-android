// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.event.entity;

/** An event about an entity. */
public abstract class EntityEvent extends net.neoforged.bus.api.Event {
    private final net.minecraft.world.entity.Entity entity;

    protected EntityEvent(net.minecraft.world.entity.Entity entity) {
        this.entity = entity;
    }

    public net.minecraft.world.entity.Entity getEntity() {
        return entity;
    }
}
