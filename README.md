# f(ile)copy - Copy the content of a file
**fcopy** is a simple and fast program, to copy the content of a file into the clipboard.
**fcopy** is programmed for the *X Window System* and currently does only support that (to an extend).

# Usage
For now, **fcopy** can only copy the whole content of a file to the clipboard:

    fcopy <file>
Simple.

# Current status
Currently **fcopy** is not really usable yet and more or less in an experimental state. That's why there are no official builds yet.
But it's working to an extend. So you could build it from source to try it out if you are really bored.

# To-do's and goals

 - [ ] Get the [ICCCM](https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html) standard as right as possible
 - [ ] Support more targets (atoms)
 - [ ] Adding better error handling
 - [ ] Refactor much of the current code
 - [ ] Adding more options
	 - [ ] *-n* for lines
	 - [ ] *-r* for regex expression (maybe)
	 - [ ] Anything that comes to my mind

