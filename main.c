#include <gtk/gtk.h>
#include <gio/gio.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <sys/sysinfo.h>
#include <glib/gstdio.h>
#include "monitor.h"
#include "power_core.h"

typedef struct {
    char distro[64];
    int cores;
    char gpu_info[256];
    char mode_name[32];
    char current_mode[32];
} SystemProfile;

SystemProfile profile;
GtkWidget *planet_buttons[5];
const char *mode_names[] = {"Ultra Perf", "Perf", "Balanced", "Save", "Ultra Save"};

const char *t_border[] = {"rgba(218,165,32,0.4)", "rgba(0,188,212,0.4)", "rgba(76,175,80,0.4)", "rgba(233,30,99,0.4)", "rgba(156,39,176,0.4)", "rgba(244,67,54,0.4)", "rgba(33,150,243,0.4)", "rgba(205,220,57,0.4)", "rgba(255,152,0,0.4)", "rgba(158,158,158,0.4)"};
const char *t_shadow[] = {"rgba(218,165,32,0.2)", "rgba(0,188,212,0.2)", "rgba(76,175,80,0.2)", "rgba(233,30,99,0.2)", "rgba(156,39,176,0.2)", "rgba(244,67,54,0.2)", "rgba(33,150,243,0.2)", "rgba(205,220,57,0.2)", "rgba(255,152,0,0.2)", "rgba(158,158,158,0.2)"};
const char *t_hover[]  = {"rgba(218,165,32,0.7)", "rgba(0,188,212,0.7)", "rgba(76,175,80,0.7)", "rgba(233,30,99,0.7)", "rgba(156,39,176,0.7)", "rgba(244,67,54,0.7)", "rgba(33,150,243,0.7)", "rgba(205,220,57,0.7)", "rgba(255,152,0,0.7)", "rgba(158,158,158,0.7)"};
const char *t_act_b[]  = {"#ffea00", "#00e5ff", "#69f0ae", "#ff4081", "#e040fb", "#ff5252", "#448aff", "#eeff41", "#ffd740", "#e0e0e0"};
const char *t_act_s[]  = {"rgba(255,234,0,0.5)", "rgba(0,229,255,0.5)", "rgba(105,240,174,0.5)", "rgba(255,64,129,0.5)", "rgba(224,64,251,0.5)", "rgba(255,82,82,0.5)", "rgba(68,138,255,0.5)", "rgba(238,255,65,0.5)", "rgba(255,215,64,0.5)", "rgba(224,224,224,0.5)"};
const char *t_act_is[] = {"rgba(255,234,0,0.2)", "rgba(0,229,255,0.2)", "rgba(105,240,174,0.2)", "rgba(255,64,129,0.2)", "rgba(224,64,251,0.2)", "rgba(255,82,82,0.2)", "rgba(68,138,255,0.2)", "rgba(238,255,65,0.2)", "rgba(255,215,64,0.2)", "rgba(224,224,224,0.2)"};
const char *t_title[]  = {"#daa520", "#00bcd4", "#4caf50", "#e91e63", "#9c27b0", "#f44336", "#2196f3", "#cddc39", "#ff9800", "#9e9e9e"};
const char *t_bg_act[] = {"#2a2a15", "#152a2a", "#152a1a", "#2a1520", "#24152a", "#2a1515", "#15202a", "#282a15", "#2a2215", "#222222"};

int current_theme = 0;
GtkCssProvider *global_provider = NULL;

const char* get_current_mode(void) {
    return profile.current_mode;
}

typedef struct {
    volatile gint ref_count;
    GtkWidget *stack;
    GtkWidget *progress_bar;
    GtkWidget *status_label;
    GtkWidget *error_details;
    GtkWidget *popup;
    double progress;
    char target_mode[32];
    char last_error_msg[256];
    guint timer_id;
    FILE *log_file;
    gboolean is_finished;
} ExecutionState;

void state_unref(ExecutionState *state) {
    if (g_atomic_int_dec_and_test(&state->ref_count)) {
        if (state->timer_id > 0) g_source_remove(state->timer_id);
        if (state->log_file) fclose(state->log_file);
        g_free(state);
    }
}

void init_config() {
    const char *config_home = g_get_user_config_dir();
    char config_path[512];
    snprintf(config_path, sizeof(config_path), "%s/pwraxiom", config_home);
    g_mkdir_with_parents(config_path, 0755);
    
    char mode_path[512];
    snprintf(mode_path, sizeof(mode_path), "%s/mode.conf", config_path);
    FILE *f = fopen(mode_path, "r");
    if (f) {
        if (fgets(profile.current_mode, sizeof(profile.current_mode), f) != NULL) {
            profile.current_mode[strcspn(profile.current_mode, "\n")] = 0;
        }
        fclose(f);
    } else {
        strcpy(profile.current_mode, "Unknown");
    }

    char theme_path[512];
    snprintf(theme_path, sizeof(theme_path), "%s/theme.conf", config_path);
    FILE *tf = fopen(theme_path, "r");
    if (tf) {
        char line[16];
        if (fgets(line, sizeof(line), tf) != NULL) {
            current_theme = atoi(line);
            if(current_theme < 0 || current_theme > 9) current_theme = 0;
        }
        fclose(tf);
    }
}

void save_config(const char *mode) {
    const char *config_home = g_get_user_config_dir();
    char config_path[512];
    snprintf(config_path, sizeof(config_path), "%s/pwraxiom/mode.conf", config_home);
    FILE *f = fopen(config_path, "w");
    if (f) {
        fprintf(f, "%s", mode);
        fclose(f);
        strcpy(profile.current_mode, mode);
    }
}

void save_theme_config(int index) {
    const char *config_home = g_get_user_config_dir();
    char config_path[512];
    snprintf(config_path, sizeof(config_path), "%s/pwraxiom/theme.conf", config_home);
    FILE *f = fopen(config_path, "w");
    if (f) {
        fprintf(f, "%d", index);
        fclose(f);
    }
}

void detect_system() {
    profile.cores = get_nprocs();
    FILE *f = fopen("/etc/os-release", "r");
    if (f) {
        char line[256];
        while (fgets(line, sizeof(line), f)) {
            if (strncmp(line, "ID=", 3) == 0) {
                strcpy(profile.distro, line + 3);
                profile.distro[strcspn(profile.distro, "\n")] = 0;
                break;
            }
        }
        fclose(f);
    }
    FILE *gpu_f = popen("lspci | grep -i 'vga\\|3d' | sed 's/.*: //'", "r");
    profile.gpu_info[0] = '\0';
    if (gpu_f) {
        char line[128];
        int count = 0;
        while (fgets(line, sizeof(line), gpu_f) && count < 2) {
            line[strcspn(line, "\n")] = 0;
            if (strlen(line) > 42) { line[39] = '.'; line[40] = '.'; line[41] = '.'; line[42] = '\0'; }
            if (strlen(profile.gpu_info) > 0) strcat(profile.gpu_info, "\n");
            strcat(profile.gpu_info, line);
            count++;
        }
        pclose(gpu_f);
    }
    if (strlen(profile.gpu_info) == 0) strcpy(profile.gpu_info, "Unknown GPU");
}

void apply_theme(int index) {
    if(!global_provider) return;
    gchar *css = g_strdup_printf(
        "window { background-color: #0a0a0a; }"
        "button.planet { border-radius: 10px; min-width: 110px; min-height: 75px; background: #151515; border: 1px solid %s; box-shadow: 0 0 15px %s; color: white; font-weight: bold; transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1); }"
        "button.planet:hover { box-shadow: 0 0 25px %s; background: #1f1f1f; }"
        "button.active-planet { border: 2px solid %s; background: %s; box-shadow: 0 0 20px 4px %s, inset 0 0 8px %s; color: %s; text-shadow: 0 0 5px %s; }"
        ".info-card { background-color: #121212; border: 1px solid #2a2a2a; border-radius: 12px; padding: 10px 14px; }"
        ".info-title { font-size: 11px; color: %s; font-weight: bold; margin-bottom: 2px; }"
        ".info-value { font-size: 13px; color: #e0e0e0; }"
        ".sidebar { background: rgba(12,12,12,0.98); border-left: 1px solid #2a2a2a; padding: 20px 15px; min-width: 220px; }"
        ".sidebar-title { color: #888; font-size: 11px; font-weight: bold; margin-bottom: 8px; }"
        "button.color-btn { border-radius: 50%%; min-width: 26px; min-height: 26px; border: 2px solid #2a2a2a; padding: 0; }"
        "button.color-btn:hover { border-color: white; }"
        "button.gear-btn { background: transparent; border: none; font-size: 24px; color: #888; transition: all 0.2s; }"
        "button.gear-btn:hover { color: %s; text-shadow: 0 0 10px %s; }"
        ".popup-title { color: %s; font-size: 24px; font-weight: bold; }"
        ".about-text { color: #888; font-size: 12px; margin-top: 15px; }"
        ".error-icon { font-size: 56px; margin-bottom: 10px; }"
        ".error-title { color: #ff4444; font-size: 20px; font-weight: bold; margin-bottom: 10px; }"
        ".error-details { color: #e0e0e0; font-size: 13px; }",
        t_border[index], t_shadow[index], t_hover[index],
        t_act_b[index], t_bg_act[index], t_act_s[index], t_act_is[index], t_act_b[index], t_act_b[index],
        t_title[index],
        t_title[index], t_shadow[index],
        t_title[index]
    );
    gtk_css_provider_load_from_string(global_provider, css);
    g_free(css);
}

void on_color_clicked(GtkWidget *btn, gpointer data) {
    int idx = GPOINTER_TO_INT(data);
    current_theme = idx;
    apply_theme(idx);
    save_theme_config(idx);
}

void toggle_sidebar(GtkWidget *btn, gpointer data) {
    GtkRevealer *rev = GTK_REVEALER(data);
    gtk_revealer_set_reveal_child(rev, !gtk_revealer_get_reveal_child(rev));
}

void close_sidebar_on_click(GtkGestureClick *gesture, int n_press, double x, double y, gpointer data) {
    GtkRevealer *rev = GTK_REVEALER(data);
    if (gtk_revealer_get_reveal_child(rev)) {
        if (x < 680) { 
            gtk_revealer_set_reveal_child(rev, FALSE);
        }
    }
}

static void on_sidebar_reveal_changed(GObject *gobject, GParamSpec *pspec, gpointer user_data) {
    GtkWidget *gear_btn = GTK_WIDGET(user_data);
    gboolean revealed = gtk_revealer_get_reveal_child(GTK_REVEALER(gobject));
    gtk_widget_set_visible(gear_btn, !revealed);
}

void update_active_button(const char *mode) {
    for (int i = 0; i < 5; i++) {
        gtk_widget_remove_css_class(planet_buttons[i], "active-planet");
        if (strcmp(mode, mode_names[i]) == 0) {
            gtk_widget_add_css_class(planet_buttons[i], "active-planet");
        }
    }
}

gboolean close_popup_delayed(gpointer data) {
    gtk_window_destroy(GTK_WINDOW(data));
    return G_SOURCE_REMOVE;
}

static gboolean update_status_from_log(gpointer user_data) {
    ExecutionState *state = user_data;
    if (state->is_finished) return G_SOURCE_REMOVE;

    if (!state->log_file) {
        state->log_file = fopen("/tmp/pwraxiom_status", "r");
        if (!state->log_file) return G_SOURCE_CONTINUE;
    }

    char line[256];
    while (fgets(line, sizeof(line), state->log_file)) {
        if (g_str_has_prefix(line, "STATUS:")) {
            gtk_label_set_text(GTK_LABEL(state->status_label), line + 7);
            state->progress += 0.10;
            if (state->progress > 0.95) state->progress = 0.95; 
            gtk_progress_bar_set_fraction(GTK_PROGRESS_BAR(state->progress_bar), state->progress);
            
            if (strstr(line, "STATUS:Done")) {
                gtk_progress_bar_set_fraction(GTK_PROGRESS_BAR(state->progress_bar), 1.0);
                gchar *done_markup = g_strdup_printf("<span size='xx-large' weight='heavy' foreground='%s'>DONE</span>", t_title[current_theme]);
                gtk_label_set_markup(GTK_LABEL(state->status_label), done_markup);
                g_free(done_markup);
                
                save_config(state->target_mode);
                update_active_button(state->target_mode);
                g_timeout_add(500, close_popup_delayed, state->popup);
                
                state->timer_id = 0;
                return G_SOURCE_REMOVE;
            }
        } else if (strlen(line) > 0) {
            strncpy(state->last_error_msg, line, sizeof(state->last_error_msg) - 1);
        }
    }
    return G_SOURCE_CONTINUE;
}

static void on_process_finished(GObject *source_object, GAsyncResult *res, gpointer user_data) {
    ExecutionState *state = user_data;
    state->is_finished = TRUE;
    
    GError *error = NULL;
    gboolean success = g_subprocess_wait_finish(G_SUBPROCESS(source_object), res, &error);
    int exit_code = 1;
    
    if (success) {
        exit_code = g_subprocess_get_exit_status(G_SUBPROCESS(source_object));
    }

    if (!success || exit_code != 0) {
        if (state->timer_id > 0) {
            g_source_remove(state->timer_id);
            state->timer_id = 0;
        }
        
        char err_msg[512];
        if (exit_code == 126 || exit_code == 127) {
            snprintf(err_msg, sizeof(err_msg), "Authentication was canceled or failed.\n(Polkit Code: %d)", exit_code);
        } else {
            snprintf(err_msg, sizeof(err_msg), "Error during hardware configuration.\n%s\n(Exit Code: %d)", state->last_error_msg, exit_code);
        }
        gtk_label_set_text(GTK_LABEL(state->error_details), err_msg);
        gtk_stack_set_visible_child_name(GTK_STACK(state->stack), "error_page");
    }

    if (state->log_file) {
        fclose(state->log_file);
        state->log_file = NULL;
    }
    g_object_unref(source_object);
    state_unref(state);
}

void run_power_command_async(ExecutionState *state) {
    system("rm -f /tmp/pwraxiom_status");
    system("touch /tmp/pwraxiom_status");

    char install_cmd[512] = "";
    if (strstr(profile.distro, "ubuntu") || strstr(profile.distro, "debian"))
        strcpy(install_cmd, "apt install -y linux-tools-$(uname -r) pciutils > /dev/null 2>&1");
    else if (strstr(profile.distro, "fedora"))
        strcpy(install_cmd, "dnf install -y kernel-tools pciutils > /dev/null 2>&1");
    else if (strstr(profile.distro, "arch"))
        strcpy(install_cmd, "pacman -S --noconfirm --needed cpupower pciutils > /dev/null 2>&1");

    char *final_script = NULL;
    if (strcmp(state->target_mode, "Ultra Save") == 0) {
        final_script = get_script_ultra_save(profile.cores, install_cmd);
    } else if (strcmp(state->target_mode, "Save") == 0) {
        final_script = get_script_save(profile.cores, install_cmd);
    } else if (strcmp(state->target_mode, "Balanced") == 0) {
        final_script = get_script_balanced(install_cmd);
    } else if (strcmp(state->target_mode, "Perf") == 0) {
        final_script = get_script_perf(install_cmd);
    } else {
        final_script = get_script_ultra_perf(profile.cores, install_cmd);
    }

    gchar *argv[] = {"pkexec", "sh", "-c", final_script, NULL};
    GError *error = NULL;
    GSubprocess *proc = g_subprocess_newv((const gchar * const *)argv, G_SUBPROCESS_FLAGS_NONE, &error);

    g_free(final_script);

    if (proc) {
        state->timer_id = g_timeout_add(150, update_status_from_log, state);
        g_subprocess_wait_async(proc, NULL, on_process_finished, state);
    } else {
        gtk_label_set_text(GTK_LABEL(state->error_details), "Failed to launch authentication prompt.");
        gtk_stack_set_visible_child_name(GTK_STACK(state->stack), "error_page");
        state_unref(state); 
    }
}

GtkWidget* create_info_card(const char *title, const char *value) {
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 2);
    gtk_widget_add_css_class(box, "info-card");
    GtkWidget *lbl_title = gtk_label_new(title);
    gtk_widget_set_halign(lbl_title, GTK_ALIGN_START);
    gtk_widget_add_css_class(lbl_title, "info-title");
    GtkWidget *lbl_value = gtk_label_new(value);
    gtk_widget_set_halign(lbl_value, GTK_ALIGN_START);
    gtk_label_set_wrap(GTK_LABEL(lbl_value), TRUE);
    gtk_widget_add_css_class(lbl_value, "info-value");
    gtk_box_append(GTK_BOX(box), lbl_title);
    gtk_box_append(GTK_BOX(box), lbl_value);
    return box;
}

void confirm_execution(GtkWidget *widget, gpointer data) {
    ExecutionState *state = (ExecutionState *)data;
    
    GtkWidget *btn_box = gtk_widget_get_parent(widget);
    GtkWidget *progress_box = gtk_widget_get_next_sibling(gtk_widget_get_prev_sibling(btn_box));
    
    gtk_widget_set_visible(btn_box, FALSE);
    gtk_widget_set_visible(progress_box, TRUE);

    run_power_command_async(state);
}

void close_popup(GtkWidget *widget, gpointer data) {
    gtk_window_destroy(GTK_WINDOW(data));
}

void show_mode_popup(GtkWindow *parent, const char *name, const char *desc);

void on_revert_clicked(GtkWidget *btn, gpointer data) {
    GtkWindow *popup = GTK_WINDOW(data);
    GtkWindow *parent = GTK_WINDOW(gtk_window_get_transient_for(popup));
    gtk_window_destroy(popup);
    show_mode_popup(parent, "Balanced", "Standard system performance:\n• Restored CPU/GPU range\n• Default power management");
}

void show_already_active_popup(GtkWindow *parent, const char *name) {
    GtkWidget *popup = gtk_window_new();
    gtk_window_set_transient_for(GTK_WINDOW(popup), parent);
    gtk_window_set_modal(GTK_WINDOW(popup), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(popup), 380, 200);
    gtk_window_set_resizable(GTK_WINDOW(popup), FALSE);

    GtkWidget *vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 15);
    gtk_widget_set_margin_start(vbox, 25);
    gtk_widget_set_margin_end(vbox, 25);
    gtk_widget_set_margin_top(vbox, 25);
    gtk_widget_set_margin_bottom(vbox, 25);
    gtk_window_set_child(GTK_WINDOW(popup), vbox);

    GtkWidget *lbl_title = gtk_label_new("Mode Active");
    gtk_widget_add_css_class(lbl_title, "popup-title");
    gtk_widget_set_halign(lbl_title, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(vbox), lbl_title);

    char desc[256];
    snprintf(desc, sizeof(desc), "The %s profile is currently running.", name);
    GtkWidget *lbl_desc = gtk_label_new(desc);
    gtk_label_set_wrap(GTK_LABEL(lbl_desc), TRUE);
    gtk_widget_set_halign(lbl_desc, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(vbox), lbl_desc);

    GtkWidget *btn_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
    gtk_widget_set_vexpand(btn_box, TRUE);
    gtk_widget_set_valign(btn_box, GTK_ALIGN_END);
    gtk_widget_set_halign(btn_box, GTK_ALIGN_END);

    GtkWidget *btn_revert = gtk_button_new_with_label("Revert to Balanced");
    GtkWidget *btn_confirm = gtk_button_new_with_label("Confirm");
    gtk_widget_add_css_class(btn_confirm, "suggested-action");

    g_signal_connect(btn_confirm, "clicked", G_CALLBACK(close_popup), popup);
    g_signal_connect(btn_revert, "clicked", G_CALLBACK(on_revert_clicked), popup);

    gtk_box_append(GTK_BOX(btn_box), btn_revert);
    gtk_box_append(GTK_BOX(btn_box), btn_confirm);
    gtk_box_append(GTK_BOX(vbox), btn_box);

    gtk_window_present(GTK_WINDOW(popup));
}

void show_mode_popup(GtkWindow *parent, const char *name, const char *desc) {
    strcpy(profile.mode_name, name);
    GtkWidget *popup = gtk_window_new();
    gtk_window_set_transient_for(GTK_WINDOW(popup), parent);
    gtk_window_set_modal(GTK_WINDOW(popup), TRUE);
    gtk_window_set_default_size(GTK_WINDOW(popup), 380, 550);
    gtk_window_set_resizable(GTK_WINDOW(popup), FALSE);

    GtkWidget *stack = gtk_stack_new();
    gtk_stack_set_transition_type(GTK_STACK(stack), GTK_STACK_TRANSITION_TYPE_CROSSFADE);
    gtk_window_set_child(GTK_WINDOW(popup), stack);

    GtkWidget *main_vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 15);
    gtk_widget_set_margin_start(main_vbox, 25);
    gtk_widget_set_margin_end(main_vbox, 25);
    gtk_widget_set_margin_top(main_vbox, 25);
    gtk_widget_set_margin_bottom(main_vbox, 25);

    GtkWidget *lbl_title = gtk_label_new(name);
    gtk_widget_add_css_class(lbl_title, "popup-title");
    gtk_widget_set_halign(lbl_title, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(main_vbox), lbl_title);

    GtkWidget *info_container = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    char core_str[32];
    snprintf(core_str, sizeof(core_str), "%d Active Threads", profile.cores);
    gtk_box_append(GTK_BOX(info_container), create_info_card("OPERATING SYSTEM", profile.distro));
    gtk_box_append(GTK_BOX(info_container), create_info_card("PROCESSOR", core_str));
    gtk_box_append(GTK_BOX(info_container), create_info_card("GRAPHICS", profile.gpu_info));
    gtk_box_append(GTK_BOX(main_vbox), info_container);

    GtkWidget *lbl_desc = gtk_label_new(desc);
    gtk_label_set_wrap(GTK_LABEL(lbl_desc), TRUE);
    gtk_widget_set_margin_top(lbl_desc, 10);
    gtk_box_append(GTK_BOX(main_vbox), lbl_desc);

    GtkWidget *progress_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_visible(progress_box, FALSE);
    GtkWidget *status_label = gtk_label_new("Waiting for authentication...");
    gtk_box_append(GTK_BOX(progress_box), status_label);
    GtkWidget *progress_bar = gtk_progress_bar_new();
    gtk_box_append(GTK_BOX(progress_box), progress_bar);
    gtk_box_append(GTK_BOX(main_vbox), progress_box);

    GtkWidget *btn_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_vexpand(btn_box, TRUE);
    gtk_widget_set_valign(btn_box, GTK_ALIGN_END);
    GtkWidget *btn_confirm = gtk_button_new_with_label("Apply & Authenticate");
    gtk_widget_add_css_class(btn_confirm, "suggested-action");
    gtk_box_append(GTK_BOX(btn_box), btn_confirm);
    GtkWidget *btn_cancel = gtk_button_new_with_label("Cancel");
    g_signal_connect(btn_cancel, "clicked", G_CALLBACK(close_popup), popup);
    gtk_box_append(GTK_BOX(btn_box), btn_cancel);
    gtk_box_append(GTK_BOX(main_vbox), btn_box);

    gtk_stack_add_named(GTK_STACK(stack), main_vbox, "main_page");

    GtkWidget *err_vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 15);
    gtk_widget_set_margin_start(err_vbox, 25);
    gtk_widget_set_margin_end(err_vbox, 25);
    gtk_widget_set_margin_top(err_vbox, 25);
    gtk_widget_set_margin_bottom(err_vbox, 25);
    gtk_widget_set_valign(err_vbox, GTK_ALIGN_CENTER);

    GtkWidget *err_icon = gtk_label_new("⚠️");
    gtk_widget_add_css_class(err_icon, "error-icon");
    gtk_box_append(GTK_BOX(err_vbox), err_icon);

    GtkWidget *err_title = gtk_label_new("Action Failed");
    gtk_widget_add_css_class(err_title, "error-title");
    gtk_box_append(GTK_BOX(err_vbox), err_title);

    GtkWidget *error_details = gtk_label_new("");
    gtk_widget_add_css_class(error_details, "error-details");
    gtk_label_set_wrap(GTK_LABEL(error_details), TRUE);
    gtk_label_set_justify(GTK_LABEL(error_details), GTK_JUSTIFY_CENTER);
    gtk_box_append(GTK_BOX(err_vbox), error_details);

    GtkWidget *btn_close_err = gtk_button_new_with_label("Close & Review");
    gtk_widget_set_margin_top(btn_close_err, 20);
    g_signal_connect(btn_close_err, "clicked", G_CALLBACK(close_popup), popup);
    gtk_box_append(GTK_BOX(err_vbox), btn_close_err);

    gtk_stack_add_named(GTK_STACK(stack), err_vbox, "error_page");

    ExecutionState *state = g_new0(ExecutionState, 1);
    state->ref_count = 2;
    state->stack = stack;
    state->progress_bar = progress_bar;
    state->status_label = status_label;
    state->error_details = error_details;
    state->popup = popup;
    state->progress = 0.0;
    state->has_error = FALSE;
    state->is_finished = FALSE;
    state->log_file = NULL;
    strcpy(state->target_mode, profile.mode_name);
    strcpy(state->last_error_msg, "Unknown system interruption.");

    g_signal_connect(btn_confirm, "clicked", G_CALLBACK(confirm_execution), state);

    gtk_window_present(GTK_WINDOW(popup));
}

void on_planet_clicked(GtkWidget *btn, gpointer data) {
    GtkWindow *parent = GTK_WINDOW(data);
    const char *label = gtk_button_get_label(GTK_BUTTON(btn));
    
    char target[32] = "";
    char desc[256] = "";

    if (strstr(label, "Ultra\nSave")) {
        strcpy(target, "Ultra Save");
        strcpy(desc, "Extreme battery preservation:\n• CPU capped to 800MHz\n• GPUs underclocked/disabled\n• Advanced PCIe ASPM enforced");
    } else if (strstr(label, "Save")) {
        strcpy(target, "Save");
        strcpy(desc, "Standard laptop battery saving:\n• Powersave governor enabled\n• Turbo boost disabled\n• Standard PCIe ASPM");
    } else if (strstr(label, "Balanced")) {
        strcpy(target, "Balanced");
        strcpy(desc, "Standard system performance:\n• Restored CPU/GPU range\n• Default power management");
    } else if (strstr(label, "Ultra\nPerf")) {
        strcpy(target, "Ultra Perf");
        strcpy(desc, "Extreme Performance Profile:\n• Min frequency locked > 1.5GHz\n• Maximum GPU clocks enforced\n• Advanced PCIe/ALPM unlocked");
    } else {
        strcpy(target, "Perf");
        strcpy(desc, "High Performance Profile:\n• Maximum CPU/GPU allowed\n• Default active cooling");
    }

    if (strcmp(profile.current_mode, target) == 0) {
        show_already_active_popup(parent, target);
    } else {
        show_mode_popup(parent, target, desc);
    }
}

gboolean on_window_close_request(GtkWindow *window, gpointer user_data) {
    gtk_widget_set_visible(GTK_WIDGET(window), FALSE);
    return TRUE; 
}

static void activate(GtkApplication *app, gpointer user_data) {
    GList *windows = gtk_application_get_windows(app);
    if (windows) {
        gtk_window_present(GTK_WINDOW(windows->data));
        return;
    }

    init_config();
    detect_system();
    
    GtkWidget *window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "Power Axiom");
    gtk_window_set_default_size(GTK_WINDOW(window), 900, 550);
    gtk_window_set_resizable(GTK_WINDOW(window), FALSE);
    
    g_signal_connect(window, "close-request", G_CALLBACK(on_window_close_request), NULL);
    
    global_provider = gtk_css_provider_new();
    
    GtkCssProvider *btn_provider = gtk_css_provider_new();
    char btn_css[2048] = "";
    for (int i = 0; i < 10; i++) {
        char cls[128];
        snprintf(cls, sizeof(cls), ".color-btn-%d { background-color: %s; }\n", i, t_title[i]);
        strcat(btn_css, cls);
    }
    gtk_css_provider_load_from_string(btn_provider, btn_css);
    gtk_style_context_add_provider_for_display(gdk_display_get_default(), GTK_STYLE_PROVIDER(btn_provider), 800);

    apply_theme(current_theme);
    gtk_style_context_add_provider_for_display(gdk_display_get_default(), GTK_STYLE_PROVIDER(global_provider), 800);

    GtkWidget *overlay = gtk_overlay_new();
    gtk_window_set_child(GTK_WINDOW(window), overlay);

    GtkWidget *draw = gtk_drawing_area_new();
    gtk_overlay_set_child(GTK_OVERLAY(overlay), draw);

    GtkWidget *monitor_widget = create_monitor_widget();
    gtk_widget_set_halign(monitor_widget, GTK_ALIGN_CENTER);
    gtk_widget_set_valign(monitor_widget, GTK_ALIGN_START);
    gtk_widget_set_margin_top(monitor_widget, 65);
    gtk_widget_set_margin_start(monitor_widget, 40);
    gtk_widget_set_margin_end(monitor_widget, 40);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), monitor_widget);

    GtkWidget *fixed = gtk_fixed_new();
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), fixed);

    GtkWidget *revealer = gtk_revealer_new();
    gtk_revealer_set_transition_type(GTK_REVEALER(revealer), GTK_REVEALER_TRANSITION_TYPE_SLIDE_LEFT);
    gtk_widget_set_halign(revealer, GTK_ALIGN_END);
    gtk_widget_set_valign(revealer, GTK_ALIGN_FILL);

    GtkWidget *sidebar_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 15);
    gtk_widget_add_css_class(sidebar_box, "sidebar");
    
    GtkWidget *theme_lbl = gtk_label_new("THEME");
    gtk_widget_add_css_class(theme_lbl, "sidebar-title");
    gtk_widget_set_halign(theme_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(sidebar_box), theme_lbl);

    GtkWidget *color_flow = gtk_flow_box_new();
    gtk_flow_box_set_max_children_per_line(GTK_FLOW_BOX(color_flow), 5);
    gtk_flow_box_set_selection_mode(GTK_FLOW_BOX(color_flow), GTK_SELECTION_NONE);
    gtk_widget_set_halign(color_flow, GTK_ALIGN_CENTER);

    for (int i=0; i<10; i++) {
        GtkWidget *cbtn = gtk_button_new();
        gtk_widget_add_css_class(cbtn, "color-btn");
        char cls_name[32];
        snprintf(cls_name, sizeof(cls_name), "color-btn-%d", i);
        gtk_widget_add_css_class(cbtn, cls_name);
        
        g_signal_connect(cbtn, "clicked", G_CALLBACK(on_color_clicked), GINT_TO_POINTER(i));
        gtk_flow_box_insert(GTK_FLOW_BOX(color_flow), cbtn, -1);
    }
    gtk_box_append(GTK_BOX(sidebar_box), color_flow);

    GtkWidget *sep = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    gtk_widget_set_margin_top(sep, 10);
    gtk_widget_set_margin_bottom(sep, 10);
    gtk_box_append(GTK_BOX(sidebar_box), sep);

    GtkWidget *about_lbl = gtk_label_new(NULL);
    gtk_label_set_markup(GTK_LABEL(about_lbl), "<span weight='bold' size='large' color='#e0e0e0'>Power Axiom</span>\n<span size='small' color='#888'>v3.0 System</span>\n\n<span size='small' color='#aaa'>Developed by MR.PROG</span>");
    gtk_label_set_justify(GTK_LABEL(about_lbl), GTK_JUSTIFY_CENTER);
    gtk_widget_add_css_class(about_lbl, "about-text");
    gtk_box_append(GTK_BOX(sidebar_box), about_lbl);

    gtk_revealer_set_child(GTK_REVEALER(revealer), sidebar_box);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), revealer);

    GtkGesture *click = gtk_gesture_click_new();
    gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(click), GTK_PHASE_CAPTURE);
    g_signal_connect(click, "pressed", G_CALLBACK(close_sidebar_on_click), revealer);
    gtk_widget_add_controller(window, GTK_EVENT_CONTROLLER(click));

    GtkWidget *gear_btn = gtk_button_new_with_label("⚙");
    gtk_widget_add_css_class(gear_btn, "gear-btn");
    gtk_widget_set_halign(gear_btn, GTK_ALIGN_END);
    gtk_widget_set_valign(gear_btn, GTK_ALIGN_START);
    gtk_widget_set_margin_top(gear_btn, 15);
    gtk_widget_set_margin_end(gear_btn, 15);
    g_signal_connect(gear_btn, "clicked", G_CALLBACK(toggle_sidebar), revealer);
    g_signal_connect(revealer, "notify::reveal-child", G_CALLBACK(on_sidebar_reveal_changed), gear_btn);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), gear_btn);

    const char *labels[] = {"Ultra\nPerf", "Perf", "Balanced", "Save", "Ultra\nSave"};
    double x_pos[] = {120, 285, 450, 615, 780}; 
    double r = 520.0, cx = 450.0, cy = 760.0;
    
    for(int i = 0; i < 5; i++) {
        GtkWidget *btn = gtk_button_new_with_label(labels[i]);
        gtk_widget_add_css_class(btn, "planet");
        planet_buttons[i] = btn;
        double dx = x_pos[i] - cx;
        double y = cy - sqrt(r*r - dx*dx);
        gtk_fixed_put(GTK_FIXED(fixed), btn, x_pos[i] - 55, y - 37.5);
        g_signal_connect(btn, "clicked", G_CALLBACK(on_planet_clicked), window);
    }

    update_active_button(profile.current_mode);
    gtk_window_present(GTK_WINDOW(window));
}

int main(int argc, char **argv) {
    g_setenv("GSK_RENDERER", "gl", TRUE);
    GtkApplication *app = gtk_application_new("com.pwraxiom.power", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}