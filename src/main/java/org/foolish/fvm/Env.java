package org.foolish.fvm;

import com.google.common.collect.Maps;

import java.util.TreeMap;
import java.util.Map;

/**
 * Environment for FVM evaluation.
 * Since branes are essentially Single Static Assignment, the environment is
 * about to look up an identifier based on the current line-scope within a brane, as well as search upward
 */
public class Env {
    class VarVersions {
        private String id;
        private TreeMap<Integer, Targoe> storage = Maps.newTreeMap();

        public VarVersions(String id) {
            this.id = id;
        }

        public void add(Targoe rhs, int line_number) {
            assert !storage.containsKey(line_number) : "Variable " + id + " already defined at line " + line_number;
            storage.put(line_number, rhs);
        }

        public Targoe get(int line_number) {
            if (line_number < 0) {
                return latest();
            }
            Map.Entry<Integer, Targoe> r = storage.lowerEntry(line_number);
            if (r != null) {
                return r.getValue();
            } else {
                return Finear.UNKNOWN;
            }
        }

        public Targoe latest() {
            if (storage.isEmpty()) return Finear.UNKNOWN;
            return storage.lastEntry().getValue();
        }
    }

    final Env parent;
    final int my_line_number;
    final Map<String, VarVersions> vars = Maps.newConcurrentMap();
    int put_count=-1;


    public Env(){
        this(null, -1);
    }

    public Env(Env parent, int my_line_number) {
        this.parent = parent;
        this.my_line_number = my_line_number;
    }

    public void noput(){
        ++put_count;
    }

    public void put(String id, Targoe value) {
        vars.computeIfAbsent(id, k -> new VarVersions(id)).add(value, ++put_count);
    }
    public void put(String id, Targoe value, int line_number) {
        assert line_number>put_count;
        put_count=line_number;
        vars.computeIfAbsent(id, k -> new VarVersions(id)).add(value, put_count);
    }

    public Targoe get(String id) {
        return get(id, -1);
    }

    public Targoe get(String id, int from_line) {
        VarVersions versions = vars.get(id);
        if (versions != null) {
            // find the greatest line number <= from_line
            Targoe value = versions.get(from_line);
            if (value != Finear.UNKNOWN)
                return value;
        }
        if (parent != null) {
            return parent.get(id, my_line_number);
        }
        return Finear.UNKNOWN;
    }
}
