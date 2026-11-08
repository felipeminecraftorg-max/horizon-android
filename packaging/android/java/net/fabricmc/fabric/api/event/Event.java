// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.fabric.api.event;

import net.minecraft.resources.ResourceLocation;

/** Fabric API's event: listeners registered, one invoker that calls them. */
public abstract class Event<T> {
    public static final ResourceLocation DEFAULT_PHASE =
            ResourceLocation.fromNamespaceAndPath("fabric", "default");
    protected volatile T invoker;

    public final T invoker() {
        return invoker;
    }

    public abstract void register(T listener);

    public void register(ResourceLocation phase, T listener) {
        register(listener);
    }

    public void addPhaseOrdering(ResourceLocation firstPhase, ResourceLocation secondPhase) {}
}
