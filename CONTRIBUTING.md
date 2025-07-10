# How do I make a contribution?

Never made an open-source contribution before? Wondering how contributions work in this project? Here's a quick run-down!

- Find an issue that you want to address or a feature that you want to add.

- Fork the repository associated with the issue to your local GitHub organization. This means that you will have a copy of the repository under `your-GitHub-username/repository-name`.

- Clone the forked repository to your local machine using `git clone https://github.com/github-username/repository-name.git`. E.g. for a repo named "xyzRepo", the user can run https://github.com/github-username/xyzRepo.git.

- Create a new branch for your fix using `git checkout -b branch-name-here`. E.g `git checkout -b main`

- Make the appropriate changes for the issue you are trying to address or the feature that you want to add.

- Use `git add insert-paths-of-changed-files-here` to add the file contents of the changed files to the "snapshot" git uses to manage the state of the project, also known as the index.

- Use `git commit -m "Insert a short message of the changes made here"` to store the contents of the index with a descriptive message.

- Push the changes to the remote repository using `git push origin branch-name-here`.

- Submit a pull request to the upstream repository.

- Title the pull request with a short description of the changes made and the issue or bug number associated with your change. For example, you can title an issue like so **"Added more log outputting to resolve #4352"**.

- In the description of the pull request, explain the changes that you made, any issues you think exist with the pull request you made, and any questions you have for the maintainer. It's OK if your pull request is not perfect (no pull request is), the reviewer will be able to help you fix any problems and improve it!

- Wait for the pull request to be reviewed by a maintainer.

- Make changes to the pull request if the reviewing maintainer recommends them.

- Celebrate your success after your pull request is merged!

# Where can I go for help?

If you need help, you can ask questions on our **discussions** tab.


# Contributing to the Solana Micro-Lending Protocol

Thank you for your interest in contributing! We welcome all contributions, from bug reports to new features.

## How to Contribute

1.  **Fork the Repository**: Create your own copy of the project to work on.
2.  **Create a New Branch**: Make a new branch for your changes.
    ```sh
    git checkout -b feature/your-feature-name
    ```
3.  **Make Your Changes**: Write your code and add or update tests as needed.
4.  **Run Tests**: Ensure all tests pass.
    ```sh
    anchor test
    ```
5.  **Commit Your Changes**: Use a clear and descriptive commit message.
6.  **Push to Your Branch**:
    ```sh
    git push origin feature/your-feature-name
    ```
7.  **Open a Pull Request**: Submit a pull request to the `main` branch of the original repository. Please provide a detailed description of your changes.

## Reporting Bugs

If you find a bug, please open an issue and include the following:
-   A clear and concise title.
-   A description of the bug and the expected behavior.
-   Steps to reproduce the bug.
-   Any relevant logs or screenshots.

## Suggesting Features

We are open to new ideas! If you have a suggestion for a new feature, please open an issue to start a discussion.