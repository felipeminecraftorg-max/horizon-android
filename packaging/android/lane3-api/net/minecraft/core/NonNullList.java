// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

import java.util.ArrayList;
import java.util.List;

/** A list that holds no nulls (a default stands for none). */
public class NonNullList<E> extends java.util.AbstractList<E> {
    private final List<E> list;
    private final E defaultValue;

    protected NonNullList(List<E> list, E defaultValue) {
        this.list = list;
        this.defaultValue = defaultValue;
    }

    public static <E> NonNullList<E> create() {
        return new NonNullList<>(new ArrayList<>(), null);
    }

    public static <E> NonNullList<E> withSize(int size, E defaultValue) {
        List<E> list = new ArrayList<>(size);
        for (int i = 0; i < size; i++) list.add(defaultValue);
        return new NonNullList<>(list, defaultValue);
    }

    @SafeVarargs
    public static <E> NonNullList<E> of(E defaultValue, E... values) {
        return new NonNullList<>(new ArrayList<>(List.of(values)), defaultValue);
    }

    @Override
    public E get(int index) {
        return list.get(index);
    }

    @Override
    public E set(int index, E value) {
        return list.set(index, value == null ? defaultValue : value);
    }

    @Override
    public void add(int index, E value) {
        list.add(index, value == null ? defaultValue : value);
    }

    @Override
    public E remove(int index) {
        return list.remove(index);
    }

    @Override
    public int size() {
        return list.size();
    }
}
