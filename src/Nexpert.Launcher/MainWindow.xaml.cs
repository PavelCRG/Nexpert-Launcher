using System.Diagnostics;
using System.IO.Compression;
using System.IO;
using System.Text.Json;
using System.Windows;
using System.Windows.Media.Imaging;
using Microsoft.Win32;

namespace Nexpert.Launcher;

public partial class MainWindow : Window
{
    private readonly string _settingsPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Nexpert", "launcher.json");
    private string? _gameRoot;

    public MainWindow() => InitializeComponent();

    private void Window_Loaded(object sender, RoutedEventArgs e)
    {
        try
        {
            var settings = File.Exists(_settingsPath) ? JsonSerializer.Deserialize<LauncherSettings>(File.ReadAllText(_settingsPath)) : null;
            GameRootBox.Text = settings?.GameRoot ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ".minecraft");
        }
        catch (Exception ex) when (ex is IOException or JsonException)
        {
            GameRootBox.Text = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ".minecraft");
        }
        RefreshInstallation();
    }

    private void ChooseFolder_Click(object sender, RoutedEventArgs e)
    {
        var dialog = new OpenFolderDialog { Title = "Choose your local Minecraft game folder", Multiselect = false };
        if (Directory.Exists(GameRootBox.Text)) dialog.InitialDirectory = GameRootBox.Text;
        if (dialog.ShowDialog(this) == true)
        {
            GameRootBox.Text = dialog.FolderName;
            RefreshInstallation();
        }
    }

    private void VersionPicker_SelectionChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
    {
        UpdateLaunchButton();
        UpdateTexturePreview();
    }

    private void PlayButton_Click(object sender, RoutedEventArgs e)
    {
        var root = _gameRoot;
        var version = VersionPicker.SelectedItem as string;
        var engine = FindEngine();
        if (root is null || version is null || engine is null)
        {
            GameHint.Text = "Build the Rust preview client with `cargo build --manifest-path engine/Cargo.toml`, then try again.";
            return;
        }

        var start = new ProcessStartInfo(engine) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(engine)! };
        start.ArgumentList.Add("--game-root"); start.ArgumentList.Add(root);
        start.ArgumentList.Add("--version"); start.ArgumentList.Add(version);
        start.ArgumentList.Add("--assets-root"); start.ArgumentList.Add(Path.Combine(root, "assets"));
        start.ArgumentList.Add("--packs-root"); start.ArgumentList.Add(Path.Combine(root, "resourcepacks"));
        try
        {
            Process.Start(start);
            GameHint.Text = $"Rust preview started for Minecraft {version}. The current build checks local resources; 3D rendering is not implemented yet.";
        }
        catch (Exception ex) when (ex is IOException or System.ComponentModel.Win32Exception or InvalidOperationException)
        {
            GameHint.Text = "Could not start the Rust preview. Build it with `cargo build --manifest-path engine/Cargo.toml`.";
        }
    }

    private void RefreshInstallation()
    {
        var selected = ResolveGameRoot(GameRootBox.Text.Trim());
        _gameRoot = selected;
        VersionPicker.ItemsSource = null;
        PackList.ItemsSource = null;

        if (selected is null)
        {
            InstallStatus.Text = "NOT CONNECTED";
            InstallStatus.Foreground = System.Windows.Media.Brushes.Orange;
            GameHint.Text = "Choose any launcher's Minecraft folder, or a parent folder containing its instances.";
            PackHint.Text = "Connect a game folder to find installed resource packs.";
            PackCount.Text = "0 FOUND";
            TexturePreview.Source = null;
            TextureInfo.Text = "Choose a version to preview its Minecraft texture.";
            UpdateLaunchButton();
            return;
        }

        GameRootBox.Text = selected;
        string[] versions;
        try
        {
            versions = Directory.GetDirectories(Path.Combine(selected, "versions"))
                .Where(path => File.Exists(Path.Combine(path, $"{Path.GetFileName(path)}.jar")))
                .Select(path => new DirectoryInfo(path).Name)
                .OrderByDescending(name => name, StringComparer.OrdinalIgnoreCase).ToArray();
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            InstallStatus.Text = "UNAVAILABLE";
            InstallStatus.Foreground = System.Windows.Media.Brushes.Orange;
            GameHint.Text = "This folder could not be read. Choose a folder you can access.";
            UpdateLaunchButton();
            return;
        }
        VersionPicker.ItemsSource = versions;
        if (versions.Length > 0) VersionPicker.SelectedIndex = 0;

        var packsPath = Path.Combine(selected, "resourcepacks");
        string[] packs;
        try
        {
            packs = Directory.Exists(packsPath)
                ? Directory.GetFileSystemEntries(packsPath).Select(Path.GetFileName).OfType<string>().OrderBy(name => name, StringComparer.OrdinalIgnoreCase).ToArray()
                : [];
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { packs = []; }
        PackList.ItemsSource = packs;
        PackCount.Text = $"{packs.Length} FOUND";
        PackHint.Text = packs.Length == 0 ? "No resource packs found in this installation." : "Found locally. Pack selection will be wired into the renderer in a later phase.";

        var ready = versions.Length > 0;
        InstallStatus.Text = ready ? "READY" : "INCOMPLETE";
        InstallStatus.Foreground = ready ? System.Windows.Media.Brushes.LightGreen : System.Windows.Media.Brushes.Orange;
        GameHint.Text = ready
            ? $"Found {versions.Length} installed version(s) and {(packs.Length == 1 ? "1 resource pack" : $"{packs.Length} resource packs")}. Local files remain in place."
            : "No installed version folders were found in this location.";
        SaveSettings(selected);
        UpdateLaunchButton();
    }

    private void UpdateLaunchButton() => PlayButton.IsEnabled = _gameRoot is not null && VersionPicker.SelectedItem is string;

    private void UpdateTexturePreview()
    {
        TexturePreview.Source = null;
        var version = VersionPicker.SelectedItem as string;
        TextureInfo.Text = "Choose a version to preview its Minecraft texture.";
        if (_gameRoot is null || version is null) return;

        try
        {
            var jarPath = Path.Combine(_gameRoot, "versions", version, $"{version}.jar");
            using var archive = ZipFile.OpenRead(jarPath);
            var texture = archive.GetEntry("assets/minecraft/textures/block/grass_block_top.png")
                ?? throw new FileNotFoundException("Grass block texture was not found in the game JAR.");
            using var stream = texture.Open();
            var bitmap = new BitmapImage();
            bitmap.BeginInit();
            bitmap.CacheOption = BitmapCacheOption.OnLoad;
            bitmap.StreamSource = stream;
            bitmap.EndInit();
            bitmap.Freeze();
            TexturePreview.Source = bitmap;
            TextureInfo.Text = $"Minecraft | {version} | Grass block top";
        }
        catch (Exception ex) when (ex is IOException or InvalidDataException)
        {
            TextureInfo.Text = "Vanilla texture sample is unavailable for this version.";
        }
    }

    private void SaveSettings(string gameRoot)
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_settingsPath)!);
            File.WriteAllText(_settingsPath, JsonSerializer.Serialize(new LauncherSettings(gameRoot)));
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            GameHint.Text = "Could not save the selected folder; you can choose it again next time.";
        }
    }

    private static string? ResolveGameRoot(string path)
    {
        try
        {
            if (HasGameFiles(path)) return Path.GetFullPath(path);
            foreach (var childName in new[] { "game", ".minecraft", "minecraft" })
            {
                var nested = Path.Combine(path, childName);
                if (HasGameFiles(nested)) return Path.GetFullPath(nested);
            }

            foreach (var containerName in new[] { "instances", "profiles" })
            {
                var container = Path.Combine(path, containerName);
                if (!Directory.Exists(container)) continue;
                foreach (var instance in Directory.GetDirectories(container))
                {
                    foreach (var childName in new[] { ".minecraft", "minecraft", "game" })
                    {
                        var nested = Path.Combine(instance, childName);
                        if (HasGameFiles(nested)) return Path.GetFullPath(nested);
                    }
                    if (HasGameFiles(instance)) return Path.GetFullPath(instance);
                }
            }
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { return null; }
        return null;
    }

    private static bool HasGameFiles(string path) => Directory.Exists(Path.Combine(path, "versions")) &&
        Directory.EnumerateDirectories(Path.Combine(path, "versions"))
            .Any(version => File.Exists(Path.Combine(version, $"{Path.GetFileName(version)}.jar")));

    private static string? FindEngine()
    {
        for (var directory = new DirectoryInfo(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
        {
            if (!Directory.Exists(Path.Combine(directory.FullName, ".git"))) continue;
            var release = Path.Combine(directory.FullName, "engine", "target", "release", "nexpert-engine.exe");
            var debug = Path.Combine(directory.FullName, "engine", "target", "debug", "nexpert-engine.exe");
            return File.Exists(release) ? release : File.Exists(debug) ? debug : null;
        }
        return null;
    }

    private sealed record LauncherSettings(string GameRoot);
}
