#include <sys/sdt.h>
#include <stdio.h>

    int main(void)
    {
      printf("Before Marker\n");
      unsigned char i=0;
	  for(i=0;;i++) {
          if(i>100) i=0;
          printf("%d\n",i);
          DTRACE_PROBE1(foo, bar, i);
      };
      printf("After Marker\n");
      return 0;
    }
