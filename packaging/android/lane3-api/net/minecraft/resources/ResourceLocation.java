// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.resources;

import java.util.Objects;

public final class ResourceLocation implements Comparable<ResourceLocation> {
    private final String namespace;
    private final String path;

    private ResourceLocation(String namespace, String path) {
        this.namespace = namespace;
        this.path = path;
    }

    public static ResourceLocation fromNamespaceAndPath(String namespace, String path) {
        return new ResourceLocation(namespace, path);
    }

    public static ResourceLocation withDefaultNamespace(String path) {
        return new ResourceLocation("minecraft", path);
    }

    public static ResourceLocation parse(String id) {
        int colon = id.indexOf(':');
        return colon < 0 ? withDefaultNamespace(id) : new ResourceLocation(id.substring(0, colon), id.substring(colon + 1));
    }

    public static ResourceLocation tryParse(String id) {
        return id == null || id.isEmpty() ? null : parse(id);
    }

    public String getNamespace() {
        return namespace;
    }

    public String getPath() {
        return path;
    }

    public ResourceLocation withPath(String path) {
        return new ResourceLocation(namespace, path);
    }

    @Override
    public String toString() {
        return namespace + ":" + path;
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof ResourceLocation r && r.namespace.equals(namespace) && r.path.equals(path);
    }

    @Override
    public int hashCode() {
        return Objects.hash(namespace, path);
    }

    @Override
    public int compareTo(ResourceLocation o) {
        int c = path.compareTo(o.path);
        return c != 0 ? c : namespace.compareTo(o.namespace);
    }
}
