

This desktop implementation uses a system of tasks to modify the state of the program. The program state is made up of a bunch of components. A task can take ownership of any number of components in order to read and modify them, similarly to a mutex. tasks that don't share ownership of properties can run in parallel. tasks can be triggered by the user.

